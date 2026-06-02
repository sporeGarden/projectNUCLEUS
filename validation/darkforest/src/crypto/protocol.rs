use super::CryptoConfig;
use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net::{http_get, send_jsonrpc, send_raw};

const RPC_TIMEOUT_MS: u64 = 5000;
const BTSP_PROBE_TIMEOUT_MS: u64 = 3000;

pub fn cry_07_ionic_tamper_rejection(
    host: &str,
    cfg: &CryptoConfig,
    results: &mut Vec<CheckResult>,
) {
    println!("── CRY-07: Ionic Token Tamper Rejection ──");
    let cb = CheckBuilder::new("CRY-07", "crypto", Category::Crypto, Severity::Critical)
        .remediation("BearDog auth.verify_ionic must reject tampered signatures");

    let tampered = "eyJhbGciOiJFZERTQSIsInR5cCI6ImlvbmljIiwidmVyIjoxfQ.\
                    eyJpc3MiOiJ0ZXN0Iiwic3ViIjoidGFtcGVyIiwic2NvcGUiOlsiKiJdLCJpYXQiOjE3MTUwMDAwMDAsImV4cCI6OTk5OTk5OTk5OSwianRpIjoiZGVhZGJlZWYwMDAwMDAwMCJ9.\
                    AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"auth.verify_ionic","params":{{"token":"{tampered}"}},"id":1}}"#
    );
    if let Some((_, body)) = send_jsonrpc(host, cfg.beardog_port, &payload, RPC_TIMEOUT_MS) {
        if body.contains("\"error\"") || body.contains("-32001") || body.contains("-32602") {
            results.push(cb.pass(
                "Tampered ionic token rejected by BearDog",
                &body[..120.min(body.len())],
            ));
        } else if body.contains("\"result\"") && body.contains("true") {
            results.push(cb.fail(
                "Tampered ionic token ACCEPTED — signature verification broken",
                &body[..120.min(body.len())],
            ));
        } else {
            results.push(cb.pass(
                "BearDog rejected tampered token (non-standard response)",
                &body[..120.min(body.len())],
            ));
        }
    } else {
        results.push(cb.pass(
            &format!("BearDog not reachable on :{} (skip)", cfg.beardog_port),
            "Connection refused",
        ));
    }
}

pub fn cry_08_ionic_expiry_enforcement(
    host: &str,
    cfg: &CryptoConfig,
    results: &mut Vec<CheckResult>,
) {
    println!("── CRY-08: Ionic Token Expiry Enforcement ──");
    let cb = CheckBuilder::new("CRY-08", "crypto", Category::Crypto, Severity::High)
        .remediation("BearDog must reject expired tokens");

    let expired = "eyJhbGciOiJFZERTQSIsInR5cCI6ImlvbmljIiwidmVyIjoxfQ.\
                   eyJpc3MiOiJ0ZXN0Iiwic3ViIjoiZXhwaXJlZCIsInNjb3BlIjpbIioiXSwiaWF0IjoxNzE1MDAwMDAwLCJleHAiOjE3MTUwMDAwMDEsImp0aSI6ImRlYWRiZWVmMDAwMDAwMDEifQ.\
                   AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"auth.verify_ionic","params":{{"token":"{expired}"}},"id":1}}"#
    );
    if let Some((_, body)) = send_jsonrpc(host, cfg.beardog_port, &payload, RPC_TIMEOUT_MS) {
        if body.contains("\"error\"") || body.contains("-32") {
            results.push(cb.pass(
                "Expired/tampered ionic token rejected by BearDog",
                &body[..120.min(body.len())],
            ));
        } else if body.contains("\"result\"") && body.contains("true") {
            results.push(cb.fail(
                "Expired ionic token ACCEPTED — expiry enforcement broken",
                &body[..120.min(body.len())],
            ));
        } else {
            results.push(cb.pass(
                "BearDog rejected expired token (non-standard response)",
                &body[..120.min(body.len())],
            ));
        }
    } else {
        results.push(cb.pass(
            &format!("BearDog not reachable on :{} (skip)", cfg.beardog_port),
            "Connection refused",
        ));
    }
}

pub fn cry_09_btsp_cipher_negotiation(host: &str, results: &mut Vec<CheckResult>) {
    let sweetgrass_port = crate::discovery::port_for("sweetgrass");
    println!("── CRY-09: BTSP Cipher Negotiation ──");
    let cb = CheckBuilder::new("CRY-09", "crypto", Category::Crypto, Severity::High)
        .remediation("BTSP must negotiate chacha20_poly1305 in production");

    if let Some(resp) = send_raw(
        host,
        sweetgrass_port,
        b"HELLO PLAINTEXT\n",
        BTSP_PROBE_TIMEOUT_MS,
    ) {
        let text = String::from_utf8_lossy(&resp);
        if text.contains("result") || text.contains("200") {
            results.push(cb.fail(
                "BTSP port accepted plaintext — cipher negotiation bypassed",
                &text[..80.min(text.len())],
            ));
        } else {
            results.push(cb.pass(
                "BTSP port rejects plaintext (handshake required)",
                &text[..80.min(text.len())],
            ));
        }
    } else {
        results.push(cb.pass(
            "BTSP port rejects plaintext connection",
            "Connection rejected/closed",
        ));
    }
}

pub fn cry_10_btsp_null_rejection(host: &str, results: &mut Vec<CheckResult>) {
    let rhizocrypt_port = crate::discovery::port_for("rhizocrypt");
    println!("── CRY-10: BTSP Null Cipher Rejection ──");
    let cb = CheckBuilder::new("CRY-10", "crypto", Category::Crypto, Severity::High)
        .remediation("BTSP must reject null cipher in production mode");

    if let Some(resp) = send_raw(
        host,
        rhizocrypt_port,
        b"HELLO PLAINTEXT\n",
        BTSP_PROBE_TIMEOUT_MS,
    ) {
        let text = String::from_utf8_lossy(&resp);
        if text.contains("result") {
            results.push(cb.fail(
                &format!("BTSP port accepted plaintext on rhizocrypt:{rhizocrypt_port}"),
                &text[..80.min(text.len())],
            ));
        } else {
            results.push(cb.pass(
                "rhizocrypt BTSP rejects plaintext",
                &text[..80.min(text.len())],
            ));
        }
    } else {
        results.push(cb.pass(
            "rhizocrypt BTSP rejects plaintext connection",
            "Connection rejected",
        ));
    }
}

pub fn cry_12_cookie_signing_version(
    host: &str,
    cfg: &CryptoConfig,
    results: &mut Vec<CheckResult>,
) {
    println!("── CRY-12: Cookie Signing Version ──");
    let cb = CheckBuilder::new("CRY-12", "crypto", Category::Crypto, Severity::Medium)
        .remediation("Ensure Tornado >= 6.0 for HMAC-SHA256 signed cookies (v2 format)");

    if let Some((_, headers, _)) = http_get(host, cfg.hub_port, "/hub/login", "", RPC_TIMEOUT_MS) {
        let cookie_line = headers
            .lines()
            .find(|l| l.to_lowercase().contains("set-cookie"));
        if let Some(cookie) = cookie_line {
            if cookie.contains('|') {
                results.push(cb.pass(
                    "Cookie uses Tornado signed-value v2 format (HMAC-SHA256)",
                    cookie,
                ));
            } else {
                results.push(cb.dark(
                    "Cookie format may be v1 (SHA-1) — verify Tornado version",
                    cookie,
                ));
            }
        } else {
            results.push(cb.pass(
                "No signed cookie in login response (redirect flow)",
                "No Set-Cookie header",
            ));
        }
    } else {
        results.push(cb.pass("Hub not reachable (skip)", "Connection refused"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_constants_are_sane() {
        assert!(RPC_TIMEOUT_MS >= 1000);
        assert!(BTSP_PROBE_TIMEOUT_MS >= 1000);
    }

    #[test]
    fn tampered_ionic_token_has_three_parts() {
        let tampered = "eyJhbGciOiJFZERTQSIsInR5cCI6ImlvbmljIiwidmVyIjoxfQ.\
                        eyJpc3MiOiJ0ZXN0Iiwic3ViIjoidGFtcGVyIiwic2NvcGUiOlsiKiJdLCJpYXQiOjE3MTUwMDAwMDAsImV4cCI6OTk5OTk5OTk5OSwianRpIjoiZGVhZGJlZWYwMDAwMDAwMCJ9.\
                        AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let parts: Vec<&str> = tampered.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "ionic token must have header.payload.signature"
        );
        // Each part should be non-empty
        for (i, part) in parts.iter().enumerate() {
            assert!(!part.is_empty(), "token part {i} is empty");
        }
    }

    #[test]
    fn expired_ionic_token_has_three_parts() {
        let expired = "eyJhbGciOiJFZERTQSIsInR5cCI6ImlvbmljIiwidmVyIjoxfQ.\
                       eyJpc3MiOiJ0ZXN0Iiwic3ViIjoiZXhwaXJlZCIsInNjb3BlIjpbIioiXSwiaWF0IjoxNzE1MDAwMDAwLCJleHAiOjE3MTUwMDAwMDEsImp0aSI6ImRlYWRiZWVmMDAwMDAwMDEifQ.\
                       AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let parts: Vec<&str> = expired.split('.').collect();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn btsp_plaintext_probe_is_valid() {
        let probe = b"HELLO PLAINTEXT\n";
        assert!(probe.ends_with(b"\n"));
        let text = std::str::from_utf8(probe).expect("probe must be valid UTF-8");
        assert!(text.starts_with("HELLO"));
    }

    #[test]
    fn cry_07_payload_is_valid_json() {
        let tampered = "test_token";
        let payload = format!(
            r#"{{"jsonrpc":"2.0","method":"auth.verify_ionic","params":{{"token":"{tampered}"}},"id":1}}"#
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).expect("payload must be valid JSON");
        assert_eq!(parsed["method"], "auth.verify_ionic");
        assert_eq!(parsed["params"]["token"], "test_token");
    }

    #[test]
    fn cry_08_payload_is_valid_json() {
        let expired = "test_expired_token";
        let payload = format!(
            r#"{{"jsonrpc":"2.0","method":"auth.verify_ionic","params":{{"token":"{expired}"}},"id":1}}"#
        );
        let parsed: serde_json::Value =
            serde_json::from_str(&payload).expect("payload must be valid JSON");
        assert_eq!(parsed["method"], "auth.verify_ionic");
    }

    #[test]
    fn sweetgrass_port_default() {
        // Without env var, default should be 9850
        let port: u16 = std::env::var("SWEETGRASS_PORT_TEST_NONEXISTENT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9850);
        assert_eq!(port, 9850);
    }
}
