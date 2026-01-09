//! Read XPT files into Polars DataFrames.

use std::path::Path;

use polars::prelude::*;

use crate::error::XptError;

/// Read an XPT file into a Polars DataFrame.
///
/// This is the primary function for reading XPT files. It automatically
/// detects the XPT version (V5 or V8) and converts the data to a DataFrame.
///
/// # Arguments
///
/// * `path` - Path to the XPT file
///
/// # Returns
///
/// A Polars DataFrame containing the XPT data.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not a valid XPT file.
///
/// # Example
///
/// ```no_run
/// use xportrs::xportrs_read;
///
/// let df = xportrs_read("data.xpt").unwrap();
/// println!("Loaded {} rows x {} columns", df.height(), df.width());
/// ```
pub fn xportrs_read<P: AsRef<Path>>(path: P) -> Result<DataFrame, XptError> {
    // Use existing polars integration
    crate::polars::read_xpt_to_dataframe(path.as_ref())
}

#[cfg(test)]
mod tests {
    // Tests will be added once we have test fixtures
}
