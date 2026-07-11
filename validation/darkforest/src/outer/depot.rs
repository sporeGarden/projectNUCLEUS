use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net;

/// ODP-01 through ODP-04: Depot (membrane.primals.eco) security audit.
///
/// Unauthorized write attempts, checksum integrity verification,
/// rate abuse detection, and path enumeration resistance.
pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    let depot_target = depot_host(target);
    check_depot_reachable(&depot_target, results);
    check_write_rejection(&depot_target, results);
    check_checksums_available(&depot_target, results);
    check_enumeration_resistance(&depot_target, results);
}

fn depot_host(target: &str) -> String {
    std::env::var("DARKFOREST_DEPOT_HOST").unwrap_or_else(|_| format!("membrane.{target}"))
}

fn check_depot_reachable(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("ODP-01", "outer.depot", Category::Network, Severity::High)
        .remediation("Verify depot is serving binary artifacts via HTTPS");

    let resp = net::http_get(target, 443, "/depot/", "", 5000);
    results.push(match resp {
        Some((code, _, _)) if code == 200 || code == 301 || code == 302 => cb.pass(
            "Depot endpoint reachable",
            &format!("GET /depot/ returned {code}"),
        ),
        Some((code, _, _)) => cb.dark(
            "Depot endpoint returned unexpected status",
            &format!("GET /depot/ returned {code}"),
        ),
        None => cb.known_gap(
            "Depot unreachable",
            &format!("Cannot connect to {target}:443"),
        ),
    });
}

fn check_write_rejection(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("ODP-02", "outer.depot", Category::Auth, Severity::Critical)
        .remediation("Depot must be read-only via Caddy — no PUT/POST/DELETE on artifact paths");

    let methods = ["PUT", "POST", "DELETE"];
    let mut all_rejected = true;
    let mut evidence = Vec::new();

    for method in &methods {
        if let Some(code) = net::http_method(target, 443, method, "/depot/test_binary", 3000) {
            if code == 405 || code == 501 || code == 403 {
                evidence.push(format!("{method}→{code} (rejected)"));
            } else {
                all_rejected = false;
                evidence.push(format!("{method}→{code} (ACCEPTED — vulnerability)"));
            }
        } else {
            evidence.push(format!("{method}→unreachable"));
        }
    }

    results.push(if all_rejected {
        cb.pass("Depot rejects write methods", &evidence.join(", "))
    } else {
        cb.fail("Depot accepts write methods", &evidence.join(", "))
    });
}

fn check_checksums_available(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("ODP-03", "outer.depot", Category::Crypto, Severity::High)
        .remediation("Ensure checksums.toml is served from depot for BLAKE3 verification");

    let resp = net::http_get(target, 443, "/depot/checksums.toml", "", 5000);
    results.push(match resp {
        Some((200, _, ref body)) if body.contains("blake3") || body.contains("sha") => cb.pass(
            "checksums.toml available with hash data",
            &format!("Served {} bytes of checksum data", body.len()),
        ),
        Some((200, _, ref body)) => cb.dark(
            "checksums.toml served but no hash fields found",
            &format!("{} bytes, inspect manually", body.len()),
        ),
        Some((code, _, _)) => cb.fail(
            "checksums.toml not available",
            &format!("GET /depot/checksums.toml returned {code}"),
        ),
        None => cb.known_gap("Cannot verify checksums", "Depot unreachable"),
    });
}

fn check_enumeration_resistance(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new(
        "ODP-04",
        "outer.depot",
        Category::InfoLeak,
        Severity::Medium,
    )
    .remediation("Disable directory listing on depot paths");

    let resp = net::http_get(target, 443, "/depot/", "", 5000);
    results.push(match resp {
        Some((_code, _, ref body)) => {
            let has_listing = body.contains("Index of") || body.contains("Directory listing");
            if has_listing {
                cb.dark(
                    "Depot directory listing enabled",
                    "Attackers can enumerate all available binaries",
                )
            } else {
                cb.pass(
                    "Depot directory listing disabled or styled",
                    "No raw directory listing indicators",
                )
            }
        }
        None => cb.known_gap("Cannot verify enumeration", "Depot unreachable"),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depot_checks_against_unreachable_do_not_panic() {
        let mut results = Vec::new();
        run("192.0.2.1", &mut results);
        assert_eq!(results.len(), 4, "should produce 4 depot checks");
        for r in &results {
            assert!(r.id.starts_with("ODP-"), "check ID should start with ODP-");
        }
    }
}
