//! Public API functions for xportrs.
//!
//! This module provides the primary user-facing API for xportrs, inspired by
//! R's xportr package. All functions work with Polars DataFrames.
//!
//! # Functions
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`xportrs_read`] | Read XPT file into DataFrame |
//! | [`xportrs_write`] | Write DataFrame to XPT file |
//! | [`xportrs_type`] | Coerce column types to match spec |
//! | [`xportrs_length`] | Apply variable lengths from spec |
//! | [`xportrs_label`] | Apply variable labels from spec |
//! | [`xportrs_format`] | Apply SAS formats from spec |
//! | [`xportrs_order`] | Reorder columns to match spec |
//! | [`xportrs_df_label`] | Set dataset label |
//! | [`xportrs_metadata`] | Bind metadata spec to DataFrame |
//! | [`xportrs`] | All-in-one pipeline |
//! | [`xportrs_validate`] | Validate DataFrame against policy |

mod df_label;
mod metadata;
mod order;
mod pipeline;
mod read;
mod transforms;
mod validate;
mod write;

// Re-export all public functions
pub use df_label::xportrs_df_label;
pub use metadata::xportrs_metadata;
pub use order::xportrs_order;
pub use pipeline::xportrs;
pub use read::xportrs_read;
pub use transforms::{xportrs_format, xportrs_label, xportrs_length, xportrs_type};
pub use validate::xportrs_validate;
pub use write::xportrs_write;
