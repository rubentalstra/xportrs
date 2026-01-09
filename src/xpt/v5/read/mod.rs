//! XPT v5 reading functionality.
//!
//! This module provides the reader implementation for XPT v5 files.

mod obs;
mod parse;
mod reader;

pub use obs::ObservationReader;
pub use parse::{XptMemberInfo, parse_header};
pub use reader::{XptFile, XptReader};
