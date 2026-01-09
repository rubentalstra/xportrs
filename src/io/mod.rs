//! High-level I/O operations for XPT files.
//!
//! This module provides the primary user-facing API for reading and writing
//! XPT files with integrated validation and metadata support. It wraps the
//! low-level [`crate::core::reader`] and [`crate::core::writer`] modules
//! with a more ergonomic interface.
//!
//! # Reading XPT Files
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::io::{read_xpt, read_xpt_with_validation};
//! use xportrs::XptVersion;
//!
//! // Simple read
//! let dataset = read_xpt(Path::new("dm.xpt")).unwrap();
//! println!("Read {} rows", dataset.num_rows());
//!
//! // Read with validation
//! let result = read_xpt_with_validation(Path::new("dm.xpt"), XptVersion::V5).unwrap();
//! if result.validation.has_issues() {
//!     eprintln!("Issues: {}", result.validation);
//! }
//! ```
//!
//! # Writing XPT Files
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::io::{write_xpt, write_xpt_validated};
//! use xportrs::{XptDataset, XptColumn, XptVersion};
//!
//! let mut dataset = XptDataset::new("DM");
//! dataset.columns.push(XptColumn::character("USUBJID", 20));
//!
//! // Simple write
//! write_xpt(Path::new("dm.xpt"), &dataset).unwrap();
//!
//! // Write with validation
//! let result = write_xpt_validated(Path::new("dm.xpt"), &dataset, XptVersion::V5).unwrap();
//! println!("Wrote {} rows", result.rows_written);
//! ```
//!
//! # Streaming API
//!
//! For large files, use the streaming API to process rows one at a time:
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::io::read_xpt_streaming;
//!
//! let mut reader = read_xpt_streaming(Path::new("large.xpt")).unwrap();
//! for observation in reader.observations() {
//!     let obs = observation.unwrap();
//!     // Process each row...
//! }
//! ```

mod reader;
mod writer;

// Re-export reader types and functions
pub use reader::{
    get_xpt_metadata, read_xpt, read_xpt_fda_compliant, read_xpt_streaming,
    read_xpt_streaming_with_options, read_xpt_with_options, read_xpt_with_validation,
    ObservationIter, ReadResult, XptReader,
};

// Re-export writer types and functions
pub use writer::{
    write_xpt, write_xpt_against_spec, write_xpt_fda_compliant, write_xpt_validated,
    write_xpt_with_options, DatasetInfo, StreamingWriter, ValidatedWriter, WriteResult,
    XptWriter, XptWriterBuilder,
};

// Re-export commonly used types from core
pub use crate::core::reader::{DatasetMeta, StreamingReader};
