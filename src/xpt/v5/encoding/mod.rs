//! Encoding utilities for XPT v5.
//!
//! This module provides encoding/decoding for numeric values (IBM float)
//! and text values in XPT v5 format.

mod ibm_float;
mod text;

pub use ibm_float::{
    SasMissingValue, decode_ibm_float, encode_ibm_float, encode_missing_value,
    identify_missing_value, is_missing_value, missing_patterns,
};
pub use text::{decode_text, encode_text, is_valid_xpt_string, truncate_utf8};
