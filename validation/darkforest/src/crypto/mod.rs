mod filesystem;
mod keys;
mod protocol;

use crate::check::*;
use crate::net::*;
use std::path::PathBuf;

pub(crate) fn gate_home() -> PathBuf {
    PathBuf::from(std::env::var("GATE_HOME")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "/home/nobody".to_string()))
}

fn cookie_secret_path() -> PathBuf {
    std::env::var("JUPYTERHUB_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| gate_home().join("jupyterhub"))
        .join("jupyterhub_cookie_secret")
}

fn sqlite_path() -> PathBuf {
    std::env::var("JUPYTERHUB_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| gate_home().join("jupyterhub"))
        .join("jupyterhub.sqlite")
}

fn cf_dir() -> PathBuf {
    std::env::var("CLOUDFLARED_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| gate_home().join(".cloudflared"))
}

pub(crate) struct CryptoConfig {
    pub hub_port: u16,
    pub beardog_port: u16,
    pub cookie_secret: PathBuf,
    pub sqlite: PathBuf,
    pub cloudflared_dir: PathBuf,
}

pub(crate) fn shannon_entropy(data: &[u8]) -> f64 {
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

pub(crate) fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
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
            .ok().and_then(|v| v.parse().ok()).unwrap_or(9100),
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
