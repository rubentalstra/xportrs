//! Metadata specification types for XPT datasets.
//!
//! This module provides types for defining variable and dataset specifications
//! that can be used to transform and validate data before writing to XPT format.
//!
//! The specification types are inspired by the R xportr package's metadata-driven
//! approach to CDISC-compliant data transformation.
//!
//! # Example
//!
//! ```
//! use xportrs::spec::{DatasetSpec, VariableSpec};
//! use xportrs::XptType;
//!
//! let spec = DatasetSpec::new("DM")
//!     .with_label("Demographics")
//!     .add_variable(
//!         VariableSpec::character("USUBJID", 20)
//!             .with_label("Unique Subject Identifier")
//!             .with_order(1)
//!     )
//!     .add_variable(
//!         VariableSpec::numeric("AGE")
//!             .with_label("Age")
//!             .with_order(2)
//!     );
//!
//! assert_eq!(spec.name, "DM");
//! assert_eq!(spec.variables.len(), 2);
//! ```

mod dataset;
mod variable;

pub use dataset::DatasetSpec;
pub use variable::{Core, VariableSpec};
