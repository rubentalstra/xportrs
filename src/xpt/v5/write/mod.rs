//! XPT v5 writing functionality.
//!
//! This module provides the writer implementation for XPT v5 files.

mod size;
mod split;
mod writer;

pub(crate) use size::estimate_file_size_gb;
pub use split::SplitWriter;
pub use writer::XptWriter;
