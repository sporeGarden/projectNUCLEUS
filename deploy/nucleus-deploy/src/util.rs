use serde_json::Value;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

struct UtcDateTime {
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
}

fn secs_since_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn utc_from_secs(secs: u64) -> UtcDateTime {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let hour = u32::try_from(rem / 3_600).unwrap_or(0);
    let minute = u32::try_from((rem % 3_600) / 60).unwrap_or(0);
    let second = u32::try_from(rem % 60).unwrap_or(0);

    let z = i64::try_from(days).unwrap_or(0) + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    UtcDateTime {
        year: u32::try_from(year).unwrap_or(1970),
        month: u32::try_from(m).unwrap_or(1),
        day: u32::try_from(d).unwrap_or(1),
        hour,
        minute,
        second,
    }
}

pub fn utc_timestamp() -> String {
    let dt = utc_from_secs(secs_since_epoch());
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    )
}

pub fn utc_compact() -> String {
    let dt = utc_from_secs(secs_since_epoch());
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    )
}

pub fn time_stamp() -> String {
    let dt = utc_from_secs(secs_since_epoch());
    format!("{:02}:{:02}:{:02}", dt.hour, dt.minute, dt.second)
}

pub fn utc_date() -> String {
    let dt = utc_from_secs(secs_since_epoch());
    format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day)
}

pub fn utc_serial() -> String {
    let dt = utc_from_secs(secs_since_epoch());
    format!("{:04}{:02}{:02}{:02}", dt.year, dt.month, dt.day, dt.hour)
}

pub fn utc_timestamp_secs() -> u64 {
    secs_since_epoch()
}

pub fn utc_date_days_ago(days: u32) -> Option<String> {
    let secs = secs_since_epoch().checked_sub(u64::from(days) * 86_400)?;
    let dt = utc_from_secs(secs);
    Some(format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day))
}

pub fn tlog(msg: &str) {
    eprintln!("[{}] {msg}", time_stamp());
}

/// Compute BLAKE3 hash of a file (in-process, no external binary).
pub async fn blake3_hash(path: &Path) -> Option<String> {
    let data = tokio::fs::read(path).await.ok()?;
    let hash = blake3::hash(&data);
    Some(hash.to_hex().to_string())
}

/// Extract a hex string from a JSON `Value` (array of bytes or string).
pub fn value_to_hex(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(arr) => arr
            .iter()
            .filter_map(|b| b.as_u64().map(|n| format!("{n:02x}")))
            .collect(),
        _ => String::new(),
    }
}

/// Convert a hex string to a `Vec<u8>`.
pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| {
            hex.get(i..i + 2)
                .and_then(|byte_str| u8::from_str_radix(byte_str, 16).ok())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_to_hex_from_string() {
        let v = Value::String("abc123".into());
        assert_eq!(value_to_hex(&v), "abc123");
    }

    #[test]
    fn value_to_hex_from_array() {
        let v = serde_json::json!([0xab, 0xcd, 0xef]);
        assert_eq!(value_to_hex(&v), "abcdef");
    }

    #[test]
    fn value_to_hex_null() {
        assert_eq!(value_to_hex(&Value::Null), "");
    }

    #[test]
    fn hex_to_bytes_roundtrip() {
        let bytes = hex_to_bytes("abcdef");
        assert_eq!(bytes, vec![0xab, 0xcd, 0xef]);
    }

    #[test]
    fn hex_to_bytes_empty() {
        assert!(hex_to_bytes("").is_empty());
    }

    #[test]
    fn utc_from_epoch_zero() {
        let dt = utc_from_secs(0);
        assert_eq!(dt.year, 1970);
        assert_eq!(dt.month, 1);
        assert_eq!(dt.day, 1);
        assert_eq!(dt.hour, 0);
        assert_eq!(dt.minute, 0);
        assert_eq!(dt.second, 0);
    }

    #[test]
    fn utc_formats_from_epoch_zero() {
        let dt = utc_from_secs(0);
        assert_eq!(
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
            ),
            "1970-01-01T00:00:00Z"
        );
        assert_eq!(
            format!(
                "{:04}{:02}{:02}-{:02}{:02}{:02}",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
            ),
            "19700101-000000"
        );
        assert_eq!(
            format!("{:04}-{:02}-{:02}", dt.year, dt.month, dt.day),
            "1970-01-01"
        );
        assert_eq!(
            format!("{:04}{:02}{:02}{:02}", dt.year, dt.month, dt.day, dt.hour),
            "1970010100"
        );
    }
}
