//! XPT v5 writing functionality.
//!
//! This module provides the writer implementation for XPT v5 files.

mod size;
mod split;
mod writer;

pub use size::{estimate_file_size, estimate_file_size_gb, max_rows_for_size};
pub use split::SplitWriter;
pub use writer::XptWriter;
