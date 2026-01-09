//! Timestamp handling for XPT v5.
//!
//! This module handles the SAS timestamp format used in XPT v5 headers.

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

/// The SAS epoch: January 1, 1960, 00:00:00 UTC.
pub const SAS_EPOCH_DAYS: i64 = -3653; // Days from Unix epoch (1970-01-01) to SAS epoch (1960-01-01)

/// Formats a timestamp for XPT v5 header records.
///
/// Format: "DDMMMYY:HH:MM:SS" (16 characters)
#[must_use]
pub fn format_sas_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%d%b%y:%H:%M:%S").to_string().to_uppercase()
}

/// Parses a SAS timestamp from an XPT v5 header.
///
/// Format: "DDMMMYY:HH:MM:SS" (16 characters)
///
/// Returns `None` if the timestamp cannot be parsed.
#[must_use]
pub fn parse_sas_timestamp(s: &str) -> Option<DateTime<Utc>> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try parsing with 2-digit year
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%d%b%y:%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }

    // Try with 4-digit year
    if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%d%b%Y:%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }

    None
}

/// Converts a [`chrono::NaiveDate`] to SAS date value (days since 1960-01-01).
#[must_use]
pub fn sas_days_since_1960(date: chrono::NaiveDate) -> i64 {
    let sas_epoch = chrono::NaiveDate::from_ymd_opt(1960, 1, 1).unwrap();
    (date - sas_epoch).num_days()
}

/// Converts a [`chrono::NaiveDateTime`] to SAS datetime value (seconds since 1960-01-01 00:00:00).
#[must_use]
pub fn sas_seconds_since_1960(dt: chrono::NaiveDateTime) -> i64 {
    let sas_epoch = chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(1960, 1, 1).unwrap(),
        chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    );
    (dt - sas_epoch).num_seconds()
}

/// Converts a [`chrono::NaiveTime`] to SAS time value (seconds since midnight).
#[must_use]
pub fn sas_seconds_since_midnight(time: chrono::NaiveTime) -> i64 {
    let midnight = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    (time - midnight).num_seconds()
}

/// Converts a SAS date value to [`chrono::NaiveDate`].
#[must_use]
pub fn date_from_sas_days(days: i64) -> Option<chrono::NaiveDate> {
    let sas_epoch = chrono::NaiveDate::from_ymd_opt(1960, 1, 1)?;
    sas_epoch.checked_add_signed(chrono::TimeDelta::try_days(days)?)
}

/// Converts a SAS datetime value to [`chrono::NaiveDateTime`].
#[must_use]
pub fn datetime_from_sas_seconds(seconds: i64) -> Option<chrono::NaiveDateTime> {
    let sas_epoch = chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(1960, 1, 1)?,
        chrono::NaiveTime::from_hms_opt(0, 0, 0)?,
    );
    sas_epoch.checked_add_signed(chrono::TimeDelta::try_seconds(seconds)?)
}

/// Converts a SAS time value to [`chrono::NaiveTime`].
#[must_use]
pub fn time_from_sas_seconds(seconds: i64) -> Option<chrono::NaiveTime> {
    if seconds < 0 || seconds >= 86400 {
        return None;
    }
    let hours = (seconds / 3600) as u32;
    let minutes = ((seconds % 3600) / 60) as u32;
    let secs = (seconds % 60) as u32;
    chrono::NaiveTime::from_hms_opt(hours, minutes, secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_sas_days() {
        let date = NaiveDate::from_ymd_opt(1960, 1, 1).unwrap();
        assert_eq!(sas_days_since_1960(date), 0);

        let date = NaiveDate::from_ymd_opt(1960, 1, 2).unwrap();
        assert_eq!(sas_days_since_1960(date), 1);

        let date = NaiveDate::from_ymd_opt(1959, 12, 31).unwrap();
        assert_eq!(sas_days_since_1960(date), -1);
    }

    #[test]
    fn test_date_roundtrip() {
        let original = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let sas_days = sas_days_since_1960(original);
        let recovered = date_from_sas_days(sas_days).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_format_timestamp() {
        let dt = Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 45).unwrap();
        let formatted = format_sas_timestamp(dt);
        assert_eq!(formatted, "15JUN24:14:30:45");
    }
}
