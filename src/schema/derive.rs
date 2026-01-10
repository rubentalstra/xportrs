//! Schema derivation from datasets and metadata.
//!
//! This module provides the logic to derive a [`SchemaPlan`] from a
//! [`DomainDataset`] with optional metadata.

use std::collections::HashMap;

use crate::agency::Agency;
use crate::config::Config;
use crate::dataset::{ColumnData, Dataset};
use crate::error::{Result, Error};
use crate::metadata::{DatasetMetadata, VariableMetadata, XptVarType};

use super::plan::{VariableSpec, DatasetSchema};

/// Derives a schema plan from a dataset and optional metadata.
///
/// This function implements the planning algorithm described in the architecture:
///
/// 1. Resolve domain identity
/// 2. Build variable map from columns
/// 3. Merge metadata (type/length/label/format/order/role)
/// 4. Determine XPT types
/// 5. Determine lengths
/// 6. Apply ordering
/// 7. Compute byte positions and `row_len`
///
/// # Errors
///
/// Returns an error if the schema cannot be derived (e.g., invalid metadata).
pub fn derive_schema_plan(
    dataset: &Dataset,
    dataset_meta: Option<&DatasetMetadata>,
    variable_meta: Option<&[VariableMetadata]>,
    agency: Option<Agency>,
    config: &Config,
) -> Result<DatasetSchema> {
    // 1. Resolve domain identity
    let domain_code = dataset_meta
        .map(|m| m.domain_code.clone())
        .unwrap_or_else(|| dataset.domain_code().to_string());

    let dataset_label = dataset_meta
        .and_then(|m| m.dataset_label.clone())
        .or_else(|| dataset.dataset_label().map(String::from));

    // Build metadata lookup
    let var_meta_map: HashMap<&str, &VariableMetadata> = variable_meta
        .unwrap_or(&[])
        .iter()
        .map(|m| (m.variable_name.as_str(), m))
        .collect();

    // 2-5. Build variable map and determine types/lengths
    let mut planned_vars: Vec<VariableSpec> = Vec::with_capacity(dataset.columns().len());

    for (idx, col) in dataset.columns().iter().enumerate() {
        let meta = var_meta_map.get(col.name());

        // 3. Determine XPT type
        let xpt_type = if let Some(m) = meta {
            m.xpt_type.unwrap_or_else(|| infer_xpt_type(col.data()))
        } else {
            infer_xpt_type(col.data())
        };

        // 4. Determine length
        let length = determine_length(col.data(), xpt_type, meta.and_then(|m| m.length), config)?;

        // Create planned variable
        let mut planned =
            VariableSpec::new(col.name().to_string(), xpt_type, length).with_source_index(idx);

        // Merge metadata
        if let Some(m) = meta {
            if let Some(ref label) = m.label {
                planned.label = truncate_to_bytes(label, 40);
            }
            if let Some(ref format) = m.format {
                planned.format = truncate_to_bytes(format, 8);
            }
            if let Some(role) = m.role {
                planned.role = Some(role);
            }
        }

        // Use column role if not in metadata
        if planned.role.is_none() {
            planned.role = col.role();
        }

        planned_vars.push(planned);
    }

    // 6. Apply ordering
    if variable_meta.is_some() {
        // Sort by order if metadata provides it
        let mut has_order = false;
        for pv in &planned_vars {
            if let Some(m) = var_meta_map.get(pv.name.as_str())
                && m.order.is_some()
            {
                has_order = true;
                break;
            }
        }

        if has_order {
            planned_vars.sort_by(|a, b| {
                let order_a = var_meta_map
                    .get(a.name.as_str())
                    .and_then(|m| m.order)
                    .unwrap_or(i32::MAX);
                let order_b = var_meta_map
                    .get(b.name.as_str())
                    .and_then(|m| m.order)
                    .unwrap_or(i32::MAX);
                order_a.cmp(&order_b)
            });
        }
    }

    // 7. Compute byte positions and row_len
    let mut plan = DatasetSchema::new(domain_code).with_label(dataset_label);
    plan.variables = planned_vars;
    plan.recalculate_positions();

    // Apply agency name normalization if auto_fix is enabled
    if config.auto_fix
        && let Some(ag) = agency
        && ag.requires_ascii_names()
    {
        plan.domain_code = plan.domain_code.to_ascii_uppercase();
        for var in &mut plan.variables {
            var.name = var.name.to_ascii_uppercase();
        }
    }

    Ok(plan)
}

/// Infers the XPT type from column data.
fn infer_xpt_type(data: &ColumnData) -> XptVarType {
    if data.is_numeric() {
        XptVarType::Numeric
    } else {
        XptVarType::Character
    }
}

/// Determines the byte length for a variable.
fn determine_length(
    data: &ColumnData,
    xpt_type: XptVarType,
    meta_length: Option<usize>,
    config: &Config,
) -> Result<usize> {
    match xpt_type {
        XptVarType::Numeric => Ok(8), // Always 8 bytes in XPT v5
        XptVarType::Character => {
            // Use metadata length if provided
            if let Some(len) = meta_length {
                // Validate against actual data if strict
                if config.strict_checks {
                    let max_observed = compute_max_string_length(data);
                    if max_observed > len {
                        return Err(Error::invalid_schema(format!(
                            "character value exceeds specified length: max observed {} > specified {}",
                            max_observed, len
                        )));
                    }
                }
                Ok(len)
            } else {
                // Compute from data
                let max_observed = compute_max_string_length(data);
                Ok(max_observed.max(1)) // Minimum 1 byte for character
            }
        }
    }
}

/// Computes the maximum byte length of string values in the data.
fn compute_max_string_length(data: &ColumnData) -> usize {
    match data {
        ColumnData::String(vals) => vals
            .iter()
            .filter_map(|v| v.as_ref())
            .map(String::len)
            .max()
            .unwrap_or(0),
        ColumnData::Bytes(vals) => vals
            .iter()
            .filter_map(|v| v.as_ref())
            .map(Vec::len)
            .max()
            .unwrap_or(0),
        _ => 0,
    }
}

/// Truncates a string to fit within a byte limit.
fn truncate_to_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }

    // Find the last valid UTF-8 boundary within max_bytes
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }

    s[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{Column, ColumnData, Dataset};

    #[test]
    fn test_derive_schema_plan() {
        let dataset = Dataset::new(
            "AE",
            vec![
                Column::new("USUBJID", ColumnData::String(vec![Some("01-001".into())])),
                Column::new("AESEQ", ColumnData::I64(vec![Some(1)])),
            ],
        )
        .unwrap();

        let config = Config::default();
        let plan = derive_schema_plan(&dataset, None, None, None, &config).unwrap();

        assert_eq!(plan.domain_code, "AE");
        assert_eq!(plan.variables.len(), 2);
        assert_eq!(plan.variables[0].name, "USUBJID");
        assert!(plan.variables[0].xpt_type.is_character());
        assert_eq!(plan.variables[1].name, "AESEQ");
        assert!(plan.variables[1].xpt_type.is_numeric());
    }

    #[test]
    fn test_truncate_to_bytes() {
        assert_eq!(truncate_to_bytes("hello", 10), "hello");
        assert_eq!(truncate_to_bytes("hello world", 5), "hello");
        // UTF-8 boundary test
        assert_eq!(truncate_to_bytes("héllo", 2), "h");
    }
}
