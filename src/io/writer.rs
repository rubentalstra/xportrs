//! High-level XPT file writing.
//!
//! This module re-exports the writer functionality from [`crate::core::writer`]
//! for convenient access through the `io` module.

pub use crate::core::writer::{
    write_xpt, write_xpt_with_options, DatasetInfo, StreamingWriter, ValidatedWriter, XptWriter,
    XptWriterBuilder,
};
