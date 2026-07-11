use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net;

/// OTR-01 through OTR-06: TLS surface audit of the outer membrane.
///
/// Checks cipher suite, certificate chain, HSTS, protocol downgrade,
/// and TLS version negotiation against the public-facing endpoint.
pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    check_tls_reachable(target, results);
    check_hsts_header(target, results);
    check_tls_version(target, results);
    check_server_header_suppressed(target, results);
    check_cert_validity(target, results);
    check_protocol_downgrade(target, results);
}

fn check_tls_reachable(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OTR-01", "outer.tls", Category::Network, Severity::Critical)
        .remediation("Verify TLS endpoint is reachable and serving valid certificates");

    let resp = net::https_get(target, "/", "", 5000);
    results.push(match resp {
        Some((code, _headers, _body)) if code > 0 => cb.pass(
            "TLS endpoint reachable",
            &format!("HTTPS GET / returned status {code}"),
        ),
        _ => cb.fail(
            "TLS endpoint unreachable",
            &format!("Could not connect to {target}:443"),
        ),
    });
}

fn check_hsts_header(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OTR-02", "outer.tls", Category::Network, Severity::High)
        .remediation("Add Strict-Transport-Security header with includeSubDomains and preload");

    let resp = net::https_get(target, "/", "", 5000);
    results.push(match resp {
        Some((_code, ref headers, _)) if headers.to_lowercase().contains("strict-transport") => {
            let has_preload = headers.to_lowercase().contains("preload");
            let has_subdomains = headers.to_lowercase().contains("includesubdomains");
            if has_preload && has_subdomains {
                cb.pass(
                    "HSTS header present with preload + includeSubDomains",
                    "Full HSTS configuration detected",
                )
            } else {
                cb.dark(
                    "HSTS header present but incomplete",
                    &format!("preload={has_preload}, includeSubDomains={has_subdomains}"),
                )
            }
        }
        Some(_) => cb.fail("No HSTS header", "Strict-Transport-Security header missing"),
        None => cb.known_gap("Cannot verify HSTS", "TLS connection failed"),
    });
}

fn check_tls_version(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OTR-03", "outer.tls", Category::Crypto, Severity::High)
        .remediation("Ensure TLS 1.2+ only — disable SSLv3, TLS 1.0, TLS 1.1");

    let info = net::tls_probe(target, 5000);
    results.push(match info {
        Some(ref ti) => {
            let version_str = ti.version.as_deref().unwrap_or("unknown");
            let cipher_str = ti.cipher.as_deref().unwrap_or("unknown");
            let is_modern = ["1.3", "1_3", "1.2", "1_2"]
                .iter()
                .any(|v| version_str.contains(v));
            if is_modern {
                cb.pass(
                    "TLS version is modern",
                    &format!("Negotiated {version_str}, cipher {cipher_str}"),
                )
            } else {
                cb.dark(
                    "TLS version may be outdated",
                    &format!("Negotiated {version_str}"),
                )
            }
        }
        None => cb.known_gap("Cannot verify TLS version", "TLS handshake failed entirely"),
    });
}

fn check_server_header_suppressed(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OTR-04", "outer.tls", Category::InfoLeak, Severity::Medium)
        .remediation("Remove Server header from responses (Caddy: header -Server)");

    let resp = net::https_get(target, "/", "", 5000);
    results.push(match resp {
        Some((_code, ref headers, _)) => {
            let has_server = headers
                .lines()
                .any(|l| l.to_lowercase().starts_with("server:"));
            if has_server {
                cb.dark(
                    "Server header exposed",
                    "Response contains Server header — software version leak",
                )
            } else {
                cb.pass("Server header suppressed", "No Server header in response")
            }
        }
        None => cb.known_gap("Cannot verify headers", "TLS connection failed"),
    });
}

fn check_cert_validity(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OTR-05", "outer.tls", Category::Crypto, Severity::Critical)
        .remediation("Ensure ACME auto-renewal is operational — test before cert expiry");

    let resp = net::https_get(target, "/", "", 5000);
    results.push(match resp {
        Some((code, _, _)) if code > 0 => cb.pass(
            "TLS certificate accepted by client",
            "HTTPS connection established — certificate chain valid for this client",
        ),
        _ => cb.fail(
            "TLS certificate issue",
            &format!("Could not establish HTTPS to {target}:443"),
        ),
    });
}

fn check_protocol_downgrade(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OTR-06", "outer.tls", Category::Network, Severity::High)
        .remediation("Ensure port 80 redirects to HTTPS — never serves content over HTTP");

    let resp = net::http_get(target, 80, "/", "", 3000);
    results.push(match resp {
        Some((code, ref headers, _)) if (300..400).contains(&code) => {
            let location = headers
                .lines()
                .find(|l| l.to_lowercase().starts_with("location:"))
                .unwrap_or("");
            if location.to_lowercase().contains("https://") {
                cb.pass(
                    "HTTP→HTTPS redirect active",
                    &format!("Port 80 returns {code} → {location}"),
                )
            } else {
                cb.dark(
                    "Port 80 redirects but not to HTTPS",
                    &format!("Redirect to: {location}"),
                )
            }
        }
        Some((code, _, _)) => cb.fail(
            "Port 80 serves content without redirect",
            &format!("HTTP GET / returned {code} — no HTTPS redirect"),
        ),
        None => cb.pass(
            "Port 80 not reachable (closed)",
            "No HTTP listener — HTTPS-only surface",
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tls_checks_against_unreachable_do_not_panic() {
        let mut results = Vec::new();
        run("192.0.2.1", &mut results);
        assert_eq!(results.len(), 6, "should produce 6 TLS checks");
        for r in &results {
            assert!(r.id.starts_with("OTR-"), "check ID should start with OTR-");
        }
    }
}
