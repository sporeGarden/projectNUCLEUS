use chrono::Local;
use serde_json::Value;
use std::path::Path;
use tokio::process::Command;

pub fn tlog(msg: &str) {
    eprintln!("[{}] {msg}", Local::now().format("%H:%M:%S"));
}

/// Compute BLAKE3 hash of a file via `b3sum`.
pub async fn blake3_hash(path: &Path) -> Option<String> {
    let output = Command::new("b3sum").arg(path).output().await.ok()?;
    let line = String::from_utf8_lossy(&output.stdout);
    line.split_whitespace().next().map(String::from)
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
}
