//! IBM floating-point encoding for XPT v5.
//!
//! XPT v5 uses IBM System/360 floating-point format (base-16 exponent).
//! This module provides safe Rust implementations for converting between
//! IEEE 754 doubles and IBM floats.
//!
//! ## Format Overview
//!
//! IBM float (8 bytes):
//! - Byte 0: Sign (1 bit) + Exponent (7 bits, excess-64, base-16)
//! - Bytes 1-7: Mantissa (56 bits, no hidden bit)
//!
//! IEEE 754 double (8 bytes):
//! - Bit 63: Sign
//! - Bits 52-62: Exponent (11 bits, excess-1023, base-2)
//! - Bits 0-51: Mantissa (52 bits, hidden bit)

/// Encodes an IEEE 754 f64 value to IBM float format.
///
/// Returns an 8-byte array in big-endian IBM format.
/// Missing values (`None`) are encoded as the SAS missing value pattern.
#[must_use]
pub fn encode_ibm_float(value: Option<f64>) -> [u8; 8] {
    match value {
        None => MISSING_PATTERN,
        Some(v) if v.is_nan() => MISSING_PATTERN,
        Some(v) => ieee_to_ibm(v),
    }
}

/// Decodes an IBM float to IEEE 754 f64.
///
/// Returns `None` if the value represents a SAS missing value.
#[must_use]
pub fn decode_ibm_float(bytes: &[u8; 8]) -> Option<f64> {
    if is_missing_value(bytes) {
        return None;
    }
    Some(ibm_to_ieee(bytes))
}

/// Checks if the bytes represent a SAS missing value.
#[must_use]
pub fn is_missing_value(bytes: &[u8; 8]) -> bool {
    // SAS missing values have specific patterns in the first byte
    // Standard missing (.) and special missing (A-Z, _) all have
    // zero mantissa bytes (bytes 1-7)
    let first_byte = bytes[0];

    // Check for standard missing or special missing patterns
    let is_missing_marker = first_byte == 0x2E  // .
        || first_byte == 0x5F  // _
        || (0x41..=0x5A).contains(&first_byte); // A-Z

    if is_missing_marker {
        // All remaining bytes must be zero
        bytes[1..].iter().all(|&b| b == 0)
    } else {
        false
    }
}

/// SAS missing value types.
///
/// In SAS, missing values can be:
/// - Standard missing (`.`)
/// - Special missing (`.A` through `.Z`)
/// - Underscore missing (`._`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SasMissingValue {
    /// Standard missing value (`.`)
    Standard,
    /// Special missing value `.A` through `.Z`
    Special(char),
    /// Underscore missing value (`._`)
    Underscore,
}

impl SasMissingValue {
    /// Returns the byte pattern for this missing value type.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 8] {
        let first_byte = match self {
            Self::Standard => 0x2E,      // .
            Self::Underscore => 0x5F,    // _
            Self::Special(c) => c as u8, // A-Z (0x41-0x5A)
        };
        [first_byte, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    }
}

impl std::fmt::Display for SasMissingValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "."),
            Self::Special(c) => write!(f, ".{}", c),
            Self::Underscore => write!(f, "._"),
        }
    }
}

/// Identifies the specific type of SAS missing value from bytes.
///
/// Returns `None` if the bytes do not represent a missing value.
#[must_use]
pub fn identify_missing_value(bytes: &[u8; 8]) -> Option<SasMissingValue> {
    if !is_missing_value(bytes) {
        return None;
    }

    let first_byte = bytes[0];
    Some(match first_byte {
        0x2E => SasMissingValue::Standard,
        0x5F => SasMissingValue::Underscore,
        b'A'..=b'Z' => SasMissingValue::Special(first_byte as char),
        _ => return None,
    })
}

/// Encodes a SAS missing value to IBM float format.
#[must_use]
pub const fn encode_missing_value(missing: SasMissingValue) -> [u8; 8] {
    missing.to_bytes()
}

/// SAS missing value patterns for reference.
pub mod missing_patterns {
    use super::SasMissingValue;

    /// Standard missing value (`.`)
    pub const MISSING: [u8; 8] = SasMissingValue::Standard.to_bytes();
    /// Missing A (`.A`)
    pub const MISSING_A: [u8; 8] = SasMissingValue::Special('A').to_bytes();
    /// Missing B (`.B`)
    pub const MISSING_B: [u8; 8] = SasMissingValue::Special('B').to_bytes();
    /// Missing C (`.C`)
    pub const MISSING_C: [u8; 8] = SasMissingValue::Special('C').to_bytes();
    /// Missing underscore (`._`)
    pub const MISSING_UNDERSCORE: [u8; 8] = SasMissingValue::Underscore.to_bytes();
}

/// SAS missing value pattern (standard '.').
const MISSING_PATTERN: [u8; 8] = [0x2E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

/// Converts IEEE 754 f64 to IBM float.
///
/// IBM float: sign(1) + exp(7, excess-64, base-16) + mantissa(56)
/// The mantissa is normalized so the first hex digit is non-zero.
fn ieee_to_ibm(value: f64) -> [u8; 8] {
    if value == 0.0 {
        return [0u8; 8];
    }

    let bits = value.to_bits();
    let sign = ((bits >> 63) & 1) as u8;
    let ieee_exp = ((bits >> 52) & 0x7FF) as i32;
    let ieee_mant = bits & 0x000F_FFFF_FFFF_FFFF;

    // Handle special cases
    if ieee_exp == 0x7FF {
        // Infinity or NaN
        return MISSING_PATTERN;
    }

    // Handle denormalized numbers
    let (exp_adj, mant_with_hidden) = if ieee_exp == 0 {
        // Denormalized: value = 2^(-1022) * (0.mantissa)
        // Find the leading bit
        let leading = 52 - ieee_mant.leading_zeros() as i32;
        if leading <= 0 {
            return [0u8; 8];
        }
        (-1022 - 52 + leading, ieee_mant << (52 - leading + 1))
    } else {
        // Normalized: value = 2^(ieee_exp - 1023) * (1.mantissa)
        (ieee_exp - 1023, ieee_mant | 0x0010_0000_0000_0000)
    };

    // Now we have: value = sign * 2^exp_adj * (mant_with_hidden / 2^52)
    // We need to convert to: sign * 16^(ibm_exp - 64) * (ibm_mant / 16^14)
    // Where 16^14 = 2^56

    // 2^exp_adj = 16^(exp_adj/4)
    // We need to align to base-16 boundaries

    // Shift mantissa to 56 bits
    let mut mant = mant_with_hidden << 3; // Now 56-bit aligned
    let mut exp2 = exp_adj + 1;  // Account for << 3 shift

    // Normalize to base-16: shift mantissa right and adjust exponent
    // until exp2 is divisible by 4
    let shift = ((4 - (exp2 & 3)) & 3) as u32;
    if shift > 0 {
        mant >>= shift;
        exp2 += shift as i32;
    }

    // Now exp2 is divisible by 4
    let ibm_exp = (exp2 / 4) + 64;

    // Check for overflow/underflow
    if ibm_exp > 127 {
        // Overflow - return maximum value
        let mut result = [0xFFu8; 8];
        result[0] = (sign << 7) | 0x7F;
        return result;
    }
    if ibm_exp < 0 {
        // Underflow
        return [0u8; 8];
    }

    // Further normalize: IBM mantissa should have a non-zero first hex digit
    // i.e., the top 4 bits of the mantissa should be non-zero
    let mut final_exp = ibm_exp;
    while (mant >> 52) == 0 && final_exp > 0 {
        mant <<= 4;
        final_exp -= 1;
    }

    if final_exp <= 0 || mant == 0 {
        return [0u8; 8];
    }

    // Build the IBM float
    let mut result = [0u8; 8];
    result[0] = (sign << 7) | (final_exp as u8 & 0x7F);

    // Pack mantissa (take top 56 bits)
    let mant_bytes = mant.to_be_bytes();
    result[1..8].copy_from_slice(&mant_bytes[1..8]);

    result
}

/// Converts IBM float to IEEE 754 f64.
fn ibm_to_ieee(bytes: &[u8; 8]) -> f64 {
    // Check for zero
    if bytes.iter().all(|&b| b == 0) {
        return 0.0;
    }

    let sign = (bytes[0] >> 7) & 1;
    let ibm_exp = (bytes[0] & 0x7F) as i32;

    // Extract 56-bit mantissa
    let mut mant: u64 = 0;
    for &b in &bytes[1..8] {
        mant = (mant << 8) | u64::from(b);
    }

    if mant == 0 {
        return 0.0;
    }

    // IBM: value = sign * 16^(ibm_exp - 64) * (mant / 2^56)
    // IEEE: value = sign * 2^(ieee_exp - 1023) * (1 + ieee_mant / 2^52)

    // 16^(ibm_exp - 64) = 2^(4 * (ibm_exp - 64))
    let exp2 = 4 * (ibm_exp - 64);

    // The IBM mantissa is mant/2^56, we need to find the leading 1 bit
    // and normalize to 1.xxx format
    let leading_zeros = mant.leading_zeros();

    // Shift to get the mantissa in position 52-55 (hidden bit at 52)
    // The mantissa is currently in bits 55..0 (56 bits total)
    // leading_zeros tells us where the first 1 bit is from bit 63

    // Position of first 1 bit from LSB: 63 - leading_zeros
    // We want the hidden bit at position 52

    let bit_pos = 63 - leading_zeros as i32; // Position of leading 1 from LSB
    let shift_needed = bit_pos - 52;

    let ieee_mant = if shift_needed > 0 {
        (mant >> shift_needed as u32) & 0x000F_FFFF_FFFF_FFFF
    } else if shift_needed < 0 {
        (mant << (-shift_needed) as u32) & 0x000F_FFFF_FFFF_FFFF
    } else {
        mant & 0x000F_FFFF_FFFF_FFFF
    };

    // Adjust exponent: exp2 accounts for 16^x, plus adjustment for mantissa position
    // Original: mant / 2^56, with leading 1 at position bit_pos
    // Normalized: 1.xxx with value 2^bit_pos * something / 2^56 = 2^(bit_pos - 56) * something
    let ieee_exp_raw = exp2 + bit_pos - 56 + 1023;

    // Check for overflow/underflow
    if ieee_exp_raw >= 2047 {
        return if sign == 1 {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    if ieee_exp_raw <= 0 {
        // Could handle denormals, but returning 0 is simpler
        return 0.0;
    }

    // Build IEEE double
    let bits = (u64::from(sign) << 63) | ((ieee_exp_raw as u64) << 52) | ieee_mant;

    f64::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        let encoded = encode_ibm_float(Some(0.0));
        assert_eq!(encoded, [0u8; 8]);

        let decoded = decode_ibm_float(&[0u8; 8]);
        assert_eq!(decoded, Some(0.0));
    }

    #[test]
    fn test_missing() {
        let encoded = encode_ibm_float(None);
        assert!(is_missing_value(&encoded));

        let decoded = decode_ibm_float(&encoded);
        assert!(decoded.is_none());
    }

    #[test]
    fn test_roundtrip_integers() {
        for &val in &[1.0, -1.0, 100.0, -100.0, 12345.0] {
            let encoded = encode_ibm_float(Some(val));
            let decoded = decode_ibm_float(&encoded).unwrap();
            let rel_error = if val != 0.0 {
                ((decoded - val) / val).abs()
            } else {
                (decoded - val).abs()
            };
            assert!(
                rel_error < 1e-10,
                "Failed for {}: got {} (rel error: {})",
                val,
                decoded,
                rel_error
            );
        }
    }

    #[test]
    fn test_roundtrip_fractions() {
        use std::f64::consts::{E, PI};
        for &val in &[0.5, 0.25, 0.125, PI, E] {
            let encoded = encode_ibm_float(Some(val));
            let decoded = decode_ibm_float(&encoded).unwrap();
            let rel_error = ((decoded - val) / val).abs();
            assert!(
                rel_error < 1e-14,
                "Failed for {}: got {} (rel error: {})",
                val,
                decoded,
                rel_error
            );
        }
    }

    #[test]
    fn test_nan_becomes_missing() {
        let encoded = encode_ibm_float(Some(f64::NAN));
        assert!(is_missing_value(&encoded));
    }
}
