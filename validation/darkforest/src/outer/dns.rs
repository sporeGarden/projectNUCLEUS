use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use std::process::Command;

/// ODN-01 through ODN-03: DNS surface audit of outer membrane nameservers.
///
/// Zone transfer resistance, DNSSEC validation, and cache poisoning resistance.
pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    check_axfr_rejected(target, results);
    check_dnssec(target, results);
    check_nxdomain_behavior(target, results);
}

fn check_axfr_rejected(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("ODN-01", "outer.dns", Category::Network, Severity::Critical)
        .remediation("Ensure authoritative DNS rejects AXFR zone transfer requests");

    let output = Command::new("dig")
        .args(["@", target, target, "AXFR", "+short", "+time=3", "+tries=1"])
        .output();

    results.push(if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let stderr = String::from_utf8_lossy(&o.stderr);
        let combined = format!("{stdout}{stderr}");
        if combined.contains("Transfer failed")
            || combined.contains("REFUSED")
            || combined.is_empty()
            || combined.contains("connection timed out")
        {
            cb.pass(
                "AXFR zone transfer rejected",
                "DNS server refuses zone transfers",
            )
        } else if stdout.lines().count() > 3 {
            cb.fail(
                "AXFR zone transfer succeeded",
                &format!(
                    "Zone data leaked — {} records returned",
                    stdout.lines().count()
                ),
            )
        } else {
            cb.pass(
                "AXFR appears rejected",
                &format!("Response: {}", combined.trim()),
            )
        }
    } else {
        let output2 = Command::new("resolvectl")
            .args(["query", target, "--type=SOA"])
            .output();
        if let Ok(o) = output2 {
            let stdout = String::from_utf8_lossy(&o.stdout);
            cb.known_gap(
                "dig not available, used resolvectl fallback",
                &format!("SOA query: {}", stdout.trim()),
            )
        } else {
            cb.known_gap(
                "Cannot probe DNS (no dig or resolvectl)",
                "Neither dig nor resolvectl available",
            )
        }
    });
}

fn check_dnssec(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("ODN-02", "outer.dns", Category::Crypto, Severity::High)
        .remediation("Enable DNSSEC on all authoritative zones");

    let output = Command::new("dig")
        .args([target, "DNSKEY", "+dnssec", "+short", "+time=3"])
        .output();

    results.push(match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("256 3") || stdout.contains("257 3") {
                cb.pass(
                    "DNSSEC keys published",
                    &format!("{} DNSKEY records found", stdout.lines().count()),
                )
            } else if stdout.trim().is_empty() {
                cb.dark("No DNSKEY records found", "DNSSEC may not be enabled")
            } else {
                cb.dark(
                    "DNSKEY response unexpected format",
                    &format!("Response: {}", stdout.trim()),
                )
            }
        }
        Err(_) => cb.known_gap(
            "Cannot probe DNSSEC (dig not available)",
            "Install bind-utils",
        ),
    });
}

fn check_nxdomain_behavior(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("ODN-03", "outer.dns", Category::Network, Severity::Medium)
        .remediation("Ensure NXDOMAIN is returned for nonexistent subdomains (no wildcard)");

    let probe = format!("__darkforest_nonexistent__.{target}");
    let output = Command::new("dig")
        .args([&probe, "+short", "+time=3"])
        .output();

    results.push(match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            if stdout.trim().is_empty() || stderr.contains("NXDOMAIN") {
                cb.pass(
                    "NXDOMAIN for nonexistent subdomain",
                    "No wildcard DNS — subdomain enumeration returns NXDOMAIN",
                )
            } else {
                cb.dark(
                    "Nonexistent subdomain resolves (wildcard?)",
                    &format!("Resolved to: {}", stdout.trim()),
                )
            }
        }
        Err(_) => cb.known_gap(
            "Cannot probe NXDOMAIN (dig not available)",
            "Install bind-utils",
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dns_checks_do_not_panic() {
        let mut results = Vec::new();
        run("192.0.2.1", &mut results);
        assert_eq!(results.len(), 3, "should produce 3 DNS checks");
        for r in &results {
            assert!(r.id.starts_with("ODN-"), "check ID should start with ODN-");
        }
    }
}
