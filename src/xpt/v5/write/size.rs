//! File size estimation for XPT v5.
//!
//! This module provides functions to estimate the size of XPT v5 files
//! before writing, useful for file splitting decisions.

use crate::schema::DatasetSchema;
use crate::xpt::v5::constants::{NAMESTR_LEN, RECORD_LEN};

/// Overhead constants for XPT v5 files.
mod overhead {
    use super::RECORD_LEN;

    /// Library header: 3 records.
    pub const LIBRARY_HEADER: usize = 3 * RECORD_LEN;
    /// Member header: 3 records.
    pub const MEMBER_HEADER: usize = 3 * RECORD_LEN;
    /// NAMESTR header: 1 record.
    pub const NAMESTR_HEADER: usize = RECORD_LEN;
    /// OBS header: 1 record.
    pub const OBS_HEADER: usize = RECORD_LEN;
}

/// Estimates the file size for a single dataset.
///
/// Returns the estimated size in bytes.
#[must_use]
pub(crate) fn estimate_file_size(plan: &DatasetSchema, nrows: usize) -> usize {
    let mut size = 0;

    // Library header
    size += overhead::LIBRARY_HEADER;

    // Member header
    size += overhead::MEMBER_HEADER;

    // NAMESTR section
    size += overhead::NAMESTR_HEADER;
    let namestr_bytes = plan.variables.len() * NAMESTR_LEN;
    let namestr_records = namestr_bytes.div_ceil(RECORD_LEN);
    size += namestr_records * RECORD_LEN;

    // OBS header
    size += overhead::OBS_HEADER;

    // Observation data
    let obs_bytes = nrows * plan.row_len;
    let obs_records = obs_bytes.div_ceil(RECORD_LEN);
    size += obs_records * RECORD_LEN;

    size
}

/// Estimates the file size in gigabytes.
#[must_use]
pub(crate) fn estimate_file_size_gb(plan: &DatasetSchema, nrows: usize) -> f64 {
    let bytes = estimate_file_size(plan, nrows);
    bytes as f64 / (1024.0 * 1024.0 * 1024.0)
}

/// Calculates the maximum number of rows that fit in a given file size.
///
/// Returns `None` if even zero rows would exceed the limit.
#[must_use]
pub(crate) fn max_rows_for_size(plan: &DatasetSchema, max_bytes: usize) -> Option<usize> {
    // Calculate fixed overhead
    let mut fixed_overhead = overhead::LIBRARY_HEADER
        + overhead::MEMBER_HEADER
        + overhead::NAMESTR_HEADER
        + overhead::OBS_HEADER;

    let namestr_bytes = plan.variables.len() * NAMESTR_LEN;
    let namestr_records = namestr_bytes.div_ceil(RECORD_LEN);
    fixed_overhead += namestr_records * RECORD_LEN;

    if fixed_overhead >= max_bytes {
        return None;
    }

    let available = max_bytes - fixed_overhead;

    if plan.row_len == 0 {
        return Some(usize::MAX); // No columns, infinite rows
    }

    // Calculate rows that fit
    // Each record is 80 bytes, and rows are packed continuously
    // So we need to account for the padding in the last record

    // Simpler approximation: available / row_len gives a good estimate
    // Actual might be slightly more due to record alignment

    Some(available / plan.row_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::plan::VariableSpec;

    #[test]
    fn test_estimate_file_size() {
        let mut plan = DatasetSchema::new("AE".into());
        plan.variables = vec![
            VariableSpec::numeric("AESEQ"),
            VariableSpec::character("USUBJID", 20),
        ];
        plan.recalculate_positions();

        let size = estimate_file_size(&plan, 100);

        // Should include headers + namestr + obs data
        assert!(size > overhead::LIBRARY_HEADER + overhead::MEMBER_HEADER);
        assert!(size > 100 * plan.row_len);
    }

    #[test]
    fn test_max_rows_for_size() {
        let mut plan = DatasetSchema::new("AE".into());
        plan.variables = vec![VariableSpec::numeric("AESEQ")];
        plan.recalculate_positions();

        // 1 MB limit
        let max_rows = max_rows_for_size(&plan, 1024 * 1024);
        assert!(max_rows.is_some());
        assert!(max_rows.unwrap() > 0);
    }
}
