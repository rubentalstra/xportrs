//! XPT v5 reading functionality.
//!
//! This module provides the reader implementation for XPT v5 files.

// Allow unused imports - these are part of complete XPT v5 API but not all are used yet
#![allow(unused_imports)]

mod obs;
mod parse;
mod reader;

pub use obs::ObservationReader;
pub use parse::{XptMemberInfo, parse_header};
pub use reader::{XptInfo, XptReader};
