//! High-level I/O operations for XPT files.
//!
//! This module provides the primary user-facing API for reading and writing
//! XPT files. It wraps the low-level [`crate::core::reader`] and
//! [`crate::core::writer`] modules with a more ergonomic interface.
//!
//! # Reading XPT Files
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::read_xpt;
//!
//! let dataset = read_xpt(Path::new("dm.xpt")).unwrap();
//! println!("Read {} rows", dataset.num_rows());
//! ```
//!
//! # Writing XPT Files
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::{write_xpt, XptDataset, XptColumn};
//!
//! let dataset = XptDataset::with_columns("DM", vec![
//!     XptColumn::character("USUBJID", 20),
//!     XptColumn::numeric("AGE"),
//! ]);
//!
//! write_xpt(Path::new("dm.xpt"), &dataset).unwrap();
//! ```
//!
//! # Streaming API
//!
//! For large files, use the streaming API to process rows one at a time:
//!
//! ```no_run
//! use std::path::Path;
//! use xportrs::read_xpt_streaming;
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
    read_xpt, read_xpt_streaming, read_xpt_streaming_with_options, read_xpt_with_options,
    DatasetMeta, ObservationIter, StreamingReader, XptReader,
};

// Re-export writer types and functions
pub use writer::{
    write_xpt, write_xpt_with_options, DatasetInfo, StreamingWriter, ValidatedWriter, XptWriter,
    XptWriterBuilder,
};
