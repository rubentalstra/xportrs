//! Encoding utilities for XPT v5.
//!
//! This module provides encoding/decoding for numeric values (IBM float)
//! and text values in XPT v5 format.

mod ibm_float;
mod text;

pub use ibm_float::{decode_ibm_float, encode_ibm_float, is_missing_value};
pub use text::{decode_text, encode_text};
