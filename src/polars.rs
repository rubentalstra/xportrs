//! Polars `DataFrame` integration.
//!
//! This module provides conversion between xportrs [`Dataset`] and Polars
//! [`DataFrame`](polars::frame::DataFrame) types.
//!
//! # Feature Flag
//!
//! This module requires the `polars` feature:
//!
//! ```toml
//! [dependencies]
//! xportrs = { version = "0.1", features = ["polars"] }
//! ```
//!
//! # Examples
//!
//! ## Converting `Dataset` to `DataFrame`
//!
//! ```ignore
//! use xportrs::{Xpt, polars::IntoDataFrame};
//!
//! let dataset = Xpt::read("ae.xpt")?;
//! let df = dataset.into_dataframe()?;
//! println!("{}", df);
//! ```
//!
//! ## Converting `DataFrame` to `Dataset`
//!
//! ```ignore
//! use xportrs::{Dataset, polars::FromDataFrame};
//! use polars::prelude::*;
//!
//! let df: DataFrame = /* ... */;
//! let dataset = Dataset::from_dataframe("AE", df)?;
//! ```

use chrono::Timelike;
use polars::prelude::*;

use crate::dataset::{Column as XptColumn, ColumnData, Dataset, DomainCode};
use crate::error::{Error, Result};

/// Extension trait for converting a [`Dataset`] into a Polars [`DataFrame`].
pub trait IntoDataFrame {
    /// Converts this dataset into a Polars `DataFrame`.
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails due to incompatible data types.
    fn into_dataframe(self) -> Result<DataFrame>;
}

/// Extension trait for creating a [`Dataset`] from a Polars [`DataFrame`].
pub trait FromDataFrame: Sized {
    /// Creates a dataset from a Polars `DataFrame`.
    ///
    /// # Arguments
    ///
    /// * `domain_code` - The domain code for the dataset (e.g., "AE", "DM")
    /// * `df` - The Polars `DataFrame` to convert
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails due to incompatible data types.
    fn from_dataframe(domain_code: impl Into<DomainCode>, df: DataFrame) -> Result<Self>;
}

impl IntoDataFrame for Dataset {
    fn into_dataframe(self) -> Result<DataFrame> {
        let mut columns: Vec<Column> = Vec::with_capacity(self.columns().len());

        for col in self.columns() {
            let series = column_data_to_series(col.name(), col.data())?;
            columns.push(series.into());
        }

        DataFrame::new(columns).map_err(|e| Error::Polars {
            message: e.to_string(),
        })
    }
}

impl FromDataFrame for Dataset {
    fn from_dataframe(domain_code: impl Into<DomainCode>, df: DataFrame) -> Result<Self> {
        let mut columns: Vec<XptColumn> = Vec::with_capacity(df.width());

        for col in df.get_columns() {
            let xpt_col = series_to_column(col)?;
            columns.push(xpt_col);
        }

        Dataset::new(domain_code, columns)
    }
}

/// Converts xportrs `ColumnData` to a Polars Series.
fn column_data_to_series(name: &str, data: &ColumnData) -> Result<Series> {
    let series = match data {
        ColumnData::F64(values) => {
            let ca: Float64Chunked = values.iter().copied().collect();
            ca.with_name(name.into()).into_series()
        }
        ColumnData::I64(values) => {
            let ca: Int64Chunked = values.iter().copied().collect();
            ca.with_name(name.into()).into_series()
        }
        ColumnData::Bool(values) => {
            let ca: BooleanChunked = values.iter().copied().collect();
            ca.with_name(name.into()).into_series()
        }
        ColumnData::String(values) => {
            let ca: StringChunked = values.iter().map(|s| s.as_deref()).collect();
            ca.with_name(name.into()).into_series()
        }
        ColumnData::Bytes(values) => {
            // Convert bytes to hex strings for Polars representation
            let strings: Vec<Option<String>> = values
                .iter()
                .map(|opt| opt.as_ref().map(|b| bytes_to_hex(b)))
                .collect();
            let ca: StringChunked = strings.iter().map(|s: &Option<String>| s.as_deref()).collect();
            ca.with_name(name.into()).into_series()
        }
        ColumnData::Date(values) => {
            // Convert NaiveDate to days since Unix epoch for Polars Date type
            let epoch = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
            let days: Vec<Option<i32>> = values
                .iter()
                .map(|opt| {
                    opt.map(|d| {
                        let days = d.signed_duration_since(epoch).num_days();
                        days as i32
                    })
                })
                .collect();
            let ca: Int32Chunked = days.iter().copied().collect();
            ca.with_name(name.into())
                .cast(&DataType::Date)
                .map_err(|e: PolarsError| Error::Polars {
                    message: e.to_string(),
                })?
        }
        ColumnData::DateTime(values) => {
            // Convert NaiveDateTime to milliseconds since Unix epoch
            let millis: Vec<Option<i64>> = values
                .iter()
                .map(|opt| opt.map(|dt| dt.and_utc().timestamp_millis()))
                .collect();
            let ca: Int64Chunked = millis.iter().copied().collect();
            ca.with_name(name.into())
                .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
                .map_err(|e: PolarsError| Error::Polars {
                    message: e.to_string(),
                })?
        }
        ColumnData::Time(values) => {
            // Convert NaiveTime to nanoseconds since midnight
            let nanos: Vec<Option<i64>> = values
                .iter()
                .map(|opt| {
                    opt.map(|t| {
                        let secs = t.num_seconds_from_midnight() as i64;
                        let nano = t.nanosecond() as i64;
                        secs * 1_000_000_000 + nano
                    })
                })
                .collect();
            let ca: Int64Chunked = nanos.iter().copied().collect();
            ca.with_name(name.into())
                .cast(&DataType::Time)
                .map_err(|e: PolarsError| Error::Polars {
                    message: e.to_string(),
                })?
        }
    };

    Ok(series)
}

/// Convert bytes to hex string (simple implementation without external dependency).
fn bytes_to_hex(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        hex.push(HEX_CHARS[(b >> 4) as usize] as char);
        hex.push(HEX_CHARS[(b & 0x0f) as usize] as char);
    }
    hex
}

/// Converts a Polars Column to an xportrs Column.
fn series_to_column(column: &Column) -> Result<XptColumn> {
    let name = column.name().as_str();
    let series = column.as_materialized_series();
    let data = series_to_column_data(series)?;
    Ok(XptColumn::new(name, data))
}

/// Converts a Polars Series to xportrs `ColumnData`.
fn series_to_column_data(series: &Series) -> Result<ColumnData> {
    let dtype = series.dtype();

    match dtype {
        DataType::Float64 => {
            let ca = series.f64().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<f64>> = ca.into_iter().collect();
            Ok(ColumnData::F64(values))
        }
        DataType::Float32 => {
            let ca = series.f32().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<f64>> = ca.into_iter().map(|v| v.map(f64::from)).collect();
            Ok(ColumnData::F64(values))
        }
        DataType::Int64 => {
            let ca = series.i64().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().collect();
            Ok(ColumnData::I64(values))
        }
        DataType::Int32 => {
            let ca = series.i32().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::Int16 => {
            let ca = series.i16().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::Int8 => {
            let ca = series.i8().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::UInt64 => {
            let ca = series.u64().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            // Convert to i64, saturating at i64::MAX
            let values: Vec<Option<i64>> = ca
                .into_iter()
                .map(|v| v.map(|u| i64::try_from(u).unwrap_or(i64::MAX)))
                .collect();
            Ok(ColumnData::I64(values))
        }
        DataType::UInt32 => {
            let ca = series.u32().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::UInt16 => {
            let ca = series.u16().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::UInt8 => {
            let ca = series.u8().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::Boolean => {
            let ca = series.bool().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<bool>> = ca.into_iter().collect();
            Ok(ColumnData::Bool(values))
        }
        DataType::String => {
            let ca = series.str().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<String>> = ca.into_iter().map(|v| v.map(String::from)).collect();
            Ok(ColumnData::String(values))
        }
        // For Date, DateTime, Time - convert to numeric values as the primary representation
        // These can be converted back when writing to XPT using the temporal utilities
        DataType::Date => {
            // Convert Date (days since epoch) to i64 for storage
            let casted = series.cast(&DataType::Int32).map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let ca = casted.i32().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().map(|v| v.map(i64::from)).collect();
            Ok(ColumnData::I64(values))
        }
        DataType::Datetime(_, _) => {
            // Convert DateTime to i64 (milliseconds since epoch)
            let casted = series
                .cast(&DataType::Datetime(TimeUnit::Milliseconds, None))
                .map_err(|e| Error::Polars {
                    message: e.to_string(),
                })?;
            let ca = casted.i64().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().collect();
            Ok(ColumnData::I64(values))
        }
        DataType::Time => {
            // Convert Time to i64 (nanoseconds since midnight)
            let casted = series.cast(&DataType::Int64).map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let ca = casted.i64().map_err(|e| Error::Polars {
                message: e.to_string(),
            })?;
            let values: Vec<Option<i64>> = ca.into_iter().collect();
            Ok(ColumnData::I64(values))
        }
        _ => Err(Error::Polars {
            message: format!("Unsupported Polars dtype for XPT conversion: {dtype}"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataset_to_dataframe() {
        let dataset = Dataset::new(
            "AE",
            vec![
                XptColumn::new(
                    "USUBJID",
                    ColumnData::String(vec![Some("01-001".into()), Some("01-002".into())]),
                ),
                XptColumn::new("AESEQ", ColumnData::I64(vec![Some(1), Some(2)])),
                XptColumn::new("AEVAL", ColumnData::F64(vec![Some(1.5), None])),
            ],
        )
        .unwrap();

        let df = dataset.into_dataframe().unwrap();
        assert_eq!(df.width(), 3);
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn test_dataframe_to_dataset() {
        let s1 = Series::new("USUBJID".into(), &["01-001", "01-002"]);
        let s2 = Series::new("AESEQ".into(), &[1i64, 2i64]);

        let df = DataFrame::new(vec![s1.into(), s2.into()]).unwrap();

        let dataset = Dataset::from_dataframe("AE", df).unwrap();
        assert_eq!(dataset.domain_code(), "AE");
        assert_eq!(dataset.ncols(), 2);
        assert_eq!(dataset.nrows(), 2);
    }

    #[test]
    fn test_roundtrip() {
        let original = Dataset::new(
            "DM",
            vec![
                XptColumn::new("STUDYID", ColumnData::String(vec![Some("STUDY001".into())])),
                XptColumn::new("AGE", ColumnData::I64(vec![Some(45)])),
            ],
        )
        .unwrap();

        let df = original.clone().into_dataframe().unwrap();
        let roundtrip = Dataset::from_dataframe("DM", df).unwrap();

        assert_eq!(roundtrip.domain_code(), original.domain_code());
        assert_eq!(roundtrip.ncols(), original.ncols());
        assert_eq!(roundtrip.nrows(), original.nrows());
    }

    #[test]
    fn test_bytes_to_hex() {
        assert_eq!(bytes_to_hex(&[0x00, 0xff, 0xab]), "00ffab");
        assert_eq!(bytes_to_hex(&[]), "");
        assert_eq!(bytes_to_hex(&[0x12, 0x34]), "1234");
    }
}
