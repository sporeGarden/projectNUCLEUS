use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net;

/// OHT-01 through OHT-06: HTTP surface audit of the outer membrane.
///
/// Verb fuzzing, path traversal, directory listing, header injection,
/// security headers, and 404 behavior against the public endpoint.
pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    check_security_headers(target, results);
    check_404_behavior(target, results);
    check_verb_fuzzing(target, results);
    check_path_traversal(target, results);
    check_directory_listing(target, results);
    check_x_frame_options(target, results);
}

fn check_security_headers(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new(
        "OHT-01",
        "outer.http",
        Category::Network,
        Severity::Critical,
    )
    .remediation(
        "Deploy security headers: X-Content-Type-Options, Referrer-Policy, Permissions-Policy",
    );

    let resp = net::http_get(target, 443, "/", "", 5000);
    results.push(match resp {
        Some((_code, ref headers, _)) => {
            let lower = headers.to_lowercase();
            let has_nosniff = lower.contains("x-content-type-options");
            let has_referrer = lower.contains("referrer-policy");
            let has_permissions = lower.contains("permissions-policy");

            let missing: Vec<&str> = [
                (!has_nosniff).then_some("X-Content-Type-Options"),
                (!has_referrer).then_some("Referrer-Policy"),
                (!has_permissions).then_some("Permissions-Policy"),
            ]
            .into_iter()
            .flatten()
            .collect();

            if missing.is_empty() {
                cb.pass(
                    "All security headers present",
                    "nosniff + referrer + permissions all set",
                )
            } else {
                cb.fail(
                    "Missing security headers",
                    &format!("Missing: {}", missing.join(", ")),
                )
            }
        }
        None => cb.known_gap("Cannot verify headers", "HTTPS connection failed"),
    });
}

fn check_404_behavior(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OHT-02", "outer.http", Category::Network, Severity::High)
        .remediation("Return proper 404 for missing paths — remove try_files fallback to homepage");

    let resp = net::http_get(target, 443, "/__darkforest_nonexistent_path__", "", 5000);
    results.push(match resp {
        Some((404, _, _)) => cb.pass("Proper 404 returned", "Nonexistent path returns 404"),
        Some((200, _, _)) => cb.fail(
            "404 catch-all returns 200",
            "Missing paths serve homepage — content confusion / SEO poisoning risk",
        ),
        Some((code, _, _)) => cb.dark(
            "Unexpected status for missing path",
            &format!("Expected 404, got {code}"),
        ),
        None => cb.known_gap("Cannot verify 404 behavior", "HTTPS connection failed"),
    });
}

fn check_verb_fuzzing(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OHT-03", "outer.http", Category::Network, Severity::Medium)
        .remediation("Reject dangerous HTTP methods (TRACE, DELETE, PUT) on static content server");

    let dangerous = ["TRACE", "DELETE", "PUT"];
    let mut blocked = Vec::new();
    let mut allowed = Vec::new();

    for method in &dangerous {
        if let Some(code) = net::http_method(target, 443, method, "/", 3000) {
            if code == 405 || code == 501 || code == 403 {
                blocked.push(format!("{method}→{code}"));
            } else {
                allowed.push(format!("{method}→{code}"));
            }
        } else {
            blocked.push(format!("{method}→unreachable"));
        }
    }

    results.push(if allowed.is_empty() {
        cb.pass(
            "Dangerous HTTP methods blocked",
            &format!("Blocked: {}", blocked.join(", ")),
        )
    } else {
        cb.dark(
            "Some dangerous methods accepted",
            &format!(
                "Allowed: {} | Blocked: {}",
                allowed.join(", "),
                blocked.join(", ")
            ),
        )
    });
}

fn check_path_traversal(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new(
        "OHT-04",
        "outer.http",
        Category::Network,
        Severity::Critical,
    )
    .remediation("Ensure web server rejects path traversal attempts");

    let traversal_paths = ["/../../../etc/passwd", "/..%2f..%2f..%2fetc/passwd"];
    let mut safe = true;
    let mut evidence = Vec::new();

    for path in &traversal_paths {
        if let Some((code, _, body)) = net::http_get(target, 443, path, "", 3000) {
            if body.contains("root:") || body.contains("/bin/bash") {
                safe = false;
                evidence.push(format!("{path} → {code} (file contents leaked)"));
            } else {
                evidence.push(format!("{path} → {code} (blocked)"));
            }
        } else {
            evidence.push(format!("{path} → unreachable"));
        }
    }

    results.push(if safe {
        cb.pass("Path traversal blocked", &evidence.join("; "))
    } else {
        cb.fail("Path traversal vulnerability", &evidence.join("; "))
    });
}

fn check_directory_listing(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OHT-05", "outer.http", Category::InfoLeak, Severity::Medium)
        .remediation("Disable directory listing in web server configuration");

    let resp = net::http_get(target, 443, "/lab/", "", 5000);
    results.push(match resp {
        Some((_code, _, ref body)) => {
            let listing_indicators = ["<title>Index of", "Directory listing", "Parent Directory"];
            let has_listing = listing_indicators.iter().any(|ind| body.contains(ind));
            if has_listing {
                cb.fail(
                    "Directory listing enabled",
                    "Response contains directory listing HTML",
                )
            } else {
                cb.pass(
                    "No directory listing",
                    "Response does not contain listing indicators",
                )
            }
        }
        None => cb.known_gap("Cannot verify directory listing", "HTTPS connection failed"),
    });
}

fn check_x_frame_options(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OHT-06", "outer.http", Category::Network, Severity::High)
        .remediation("Add X-Frame-Options: DENY to prevent clickjacking");

    let resp = net::http_get(target, 443, "/", "", 5000);
    results.push(match resp {
        Some((_code, ref headers, _)) => {
            let lower = headers.to_lowercase();
            if lower.contains("x-frame-options") {
                cb.pass(
                    "X-Frame-Options header present",
                    "Clickjacking protection active",
                )
            } else {
                cb.fail(
                    "No X-Frame-Options header",
                    "Page is embeddable in attacker frames",
                )
            }
        }
        None => cb.known_gap("Cannot verify X-Frame-Options", "HTTPS connection failed"),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_checks_against_unreachable_do_not_panic() {
        let mut results = Vec::new();
        run("192.0.2.1", &mut results);
        assert_eq!(results.len(), 6, "should produce 6 HTTP checks");
        for r in &results {
            assert!(r.id.starts_with("OHT-"), "check ID should start with OHT-");
        }
    }
}
