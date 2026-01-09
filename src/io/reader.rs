//! High-level XPT file reading.
//!
//! This module re-exports the reader functionality from [`crate::core::reader`]
//! for convenient access through the `io` module.

pub use crate::core::reader::{
    read_xpt, read_xpt_streaming, read_xpt_streaming_with_options, read_xpt_with_options,
    DatasetMeta, ObservationIter, StreamingReader, XptReader,
};
