mod filesystem;
mod keys;
mod protocol;

use crate::check::{CheckResult, hub_port};
use std::path::PathBuf;

pub fn gate_home() -> PathBuf {
    PathBuf::from(
        std::env::var("GATE_HOME")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| "/home/nobody".to_string()),
    )
}

fn cookie_secret_path() -> PathBuf {
    std::env::var("JUPYTERHUB_DIR")
        .map_or_else(|_| gate_home().join("jupyterhub"), PathBuf::from)
        .join("jupyterhub_cookie_secret")
}

fn sqlite_path() -> PathBuf {
    std::env::var("JUPYTERHUB_DIR")
        .map_or_else(|_| gate_home().join("jupyterhub"), PathBuf::from)
        .join("jupyterhub.sqlite")
}

fn cf_dir() -> PathBuf {
    std::env::var("CLOUDFLARED_DIR")
        .map_or_else(|_| gate_home().join(".cloudflared"), PathBuf::from)
}

pub struct CryptoConfig {
    pub hub_port: u16,
    pub beardog_port: u16,
    pub cookie_secret: PathBuf,
    pub sqlite: PathBuf,
    pub cloudflared_dir: PathBuf,
}

pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut entropy = 0.0f64;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&s[i..i + 2], 16).ok()?);
    }
    Some(bytes)
}

pub fn run(host: &str, results: &mut Vec<CheckResult>) {
    let cfg = CryptoConfig {
        hub_port: hub_port(),
        beardog_port: std::env::var("BEARDOG_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9100),
        cookie_secret: cookie_secret_path(),
        sqlite: sqlite_path(),
        cloudflared_dir: cf_dir(),
    };
    println!("\n══ Crypto Strength Validation ══\n");

    filesystem::cry_01_cookie_entropy(&cfg, results);
    filesystem::cry_02_cookie_age(&cfg, results);
    filesystem::cry_03_cookie_permissions(&cfg, results);
    keys::cry_04_api_token_entropy(results);
    keys::cry_05_shadow_hash_algorithm(results);
    keys::cry_06_shadow_hash_strength(results);
    protocol::cry_07_ionic_tamper_rejection(host, &cfg, results);
    protocol::cry_08_ionic_expiry_enforcement(host, &cfg, results);
    protocol::cry_09_btsp_cipher_negotiation(host, results);
    protocol::cry_10_btsp_null_rejection(host, results);
    keys::cry_11_master_key_present(results);
    protocol::cry_12_cookie_signing_version(host, &cfg, results);
    filesystem::cry_13_sensitive_file_permissions(&cfg, results);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_empty_data_is_zero() {
        assert!((shannon_entropy(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn entropy_single_byte_repeated_is_zero() {
        assert!((shannon_entropy(&[0xAA; 256]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn entropy_uniform_distribution_is_eight() {
        let data: Vec<u8> = (0..=255).collect();
        let e = shannon_entropy(&data);
        assert!(
            (e - 8.0).abs() < 0.001,
            "uniform 256 values should yield ~8 bits, got {e}"
        );
    }

    #[test]
    fn entropy_two_values_is_one_bit() {
        let data = [0u8, 1, 0, 1, 0, 1, 0, 1];
        let e = shannon_entropy(&data);
        assert!(
            (e - 1.0).abs() < 0.001,
            "two equiprobable values should yield ~1 bit, got {e}"
        );
    }

    #[test]
    fn entropy_high_quality_random_above_threshold() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let data: Vec<u8> = (0..1024_u32)
            .map(|i| (i.wrapping_mul(37).wrapping_add(13)) as u8)
            .collect();
        assert!(
            shannon_entropy(&data) > 4.0,
            "pseudo-random data should have >4 bits entropy"
        );
    }

    #[test]
    fn hex_decode_valid_lowercase() {
        assert_eq!(hex_decode("deadbeef"), Some(vec![0xDE, 0xAD, 0xBE, 0xEF]));
    }

    #[test]
    fn hex_decode_valid_uppercase() {
        assert_eq!(hex_decode("CAFEBABE"), Some(vec![0xCA, 0xFE, 0xBA, 0xBE]));
    }

    #[test]
    fn hex_decode_valid_mixed_case() {
        assert_eq!(hex_decode("aAbBcC"), Some(vec![0xAA, 0xBB, 0xCC]));
    }

    #[test]
    fn hex_decode_empty_string() {
        assert_eq!(hex_decode(""), Some(vec![]));
    }

    #[test]
    fn hex_decode_odd_length_returns_none() {
        assert_eq!(hex_decode("abc"), None);
    }

    #[test]
    fn hex_decode_invalid_chars_returns_none() {
        assert_eq!(hex_decode("zzzz"), None);
    }

    #[test]
    fn hex_decode_trims_whitespace() {
        assert_eq!(hex_decode("  ff  "), Some(vec![0xFF]));
    }

    #[test]
    fn gate_home_returns_nonempty_path() {
        let home = gate_home();
        assert!(!home.as_os_str().is_empty());
    }
}
