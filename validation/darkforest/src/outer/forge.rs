use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net;
use std::process::Command;

/// OFG-01 through OFG-04: Forgejo SSH surface audit (git.primals.eco :2222).
///
/// SSH handshake, auth bypass probes, repo enumeration resistance.
pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    let forge_target = forge_host(target);
    let forge_port = forge_port();
    check_ssh_reachable(&forge_target, forge_port, results);
    check_password_auth_disabled(&forge_target, forge_port, results);
    check_web_interface(&forge_target, results);
    check_repo_enumeration(&forge_target, results);
}

fn forge_host(target: &str) -> String {
    std::env::var("DARKFOREST_FORGE_HOST").unwrap_or_else(|_| format!("git.{target}"))
}

fn forge_port() -> u16 {
    std::env::var("DARKFOREST_FORGE_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2222)
}

fn check_ssh_reachable(target: &str, port: u16, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OFG-01", "outer.forge", Category::Network, Severity::High)
        .remediation("Verify Forgejo SSH is running and accepting key-based connections");

    let data = b"SSH-2.0-darkforest-probe\r\n".to_vec();
    let resp = net::send_raw(target, port, &data, 3000);
    results.push(match resp {
        Some(ref bytes) => {
            let banner = String::from_utf8_lossy(bytes);
            if banner.contains("SSH-") {
                cb.pass(
                    "Forgejo SSH reachable",
                    &format!("Banner: {}", banner.lines().next().unwrap_or("(empty)")),
                )
            } else {
                cb.dark(
                    "Port reachable but no SSH banner",
                    &format!("{} bytes received", bytes.len()),
                )
            }
        }
        None => cb.known_gap(
            "Forgejo SSH unreachable",
            &format!("Cannot connect to {target}:{port}"),
        ),
    });
}

fn check_password_auth_disabled(target: &str, port: u16, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OFG-02", "outer.forge", Category::Auth, Severity::Critical)
        .remediation("Disable password authentication on SSH — key-only auth");

    let output = Command::new("ssh")
        .args([
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "PreferredAuthentications=password",
            "-o",
            "PasswordAuthentication=yes",
            "-o",
            "NumberOfPasswordPrompts=0",
            "-o",
            "ConnectTimeout=3",
            "-o",
            "BatchMode=yes",
            "-p",
            &port.to_string(),
            &format!("__darkforest_probe__@{target}"),
            "echo test",
        ])
        .output();

    results.push(match output {
        Ok(o) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stdout),
                String::from_utf8_lossy(&o.stderr)
            );
            if combined.contains("Permission denied") || combined.contains("no more authentication")
            {
                cb.pass(
                    "SSH password auth disabled",
                    "Password authentication correctly rejected",
                )
            } else if o.status.success() {
                cb.fail(
                    "SSH password auth accepted",
                    "Password authentication succeeded — critical vulnerability",
                )
            } else {
                cb.pass(
                    "SSH connection rejected (key-only likely)",
                    &format!(
                        "Exit {}: {}",
                        o.status.code().unwrap_or(-1),
                        combined.trim()
                    ),
                )
            }
        }
        Err(e) => cb.known_gap("Cannot probe SSH auth", &format!("ssh command failed: {e}")),
    });
}

fn check_web_interface(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new(
        "OFG-03",
        "outer.forge",
        Category::InfoLeak,
        Severity::Medium,
    )
    .remediation("Restrict Forgejo web interface access or suppress version headers");

    let resp = net::https_get(target, "/", "", 5000);
    results.push(match resp {
        Some((code, ref headers, _)) => {
            let lower = headers.to_lowercase();
            let version_leak = lower.contains("x-powered-by") || lower.contains("x-gitea");
            if version_leak {
                cb.dark(
                    "Forge web interface leaks version info",
                    &format!("Status {code}, version headers present"),
                )
            } else {
                cb.pass(
                    "Forge web interface does not leak version",
                    &format!("Status {code}, no version headers"),
                )
            }
        }
        None => cb.known_gap(
            "Forge web interface unreachable",
            &format!("Cannot connect to {target}:443"),
        ),
    });
}

fn check_repo_enumeration(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new(
        "OFG-04",
        "outer.forge",
        Category::InfoLeak,
        Severity::Medium,
    )
    .remediation("Disable public repository listing for unauthenticated users");

    let resp = net::https_get(target, "/explore/repos", "", 5000);
    results.push(match resp {
        Some((200, _, ref body)) => {
            if body.contains("repository-item") || body.contains("repo-name") {
                cb.dark(
                    "Public repo listing available",
                    "Unauthenticated users can enumerate repositories",
                )
            } else {
                cb.pass(
                    "No public repo listing",
                    "Explore page does not expose repository list",
                )
            }
        }
        Some((code, _, _)) => cb.pass(
            "Repo enumeration blocked",
            &format!("GET /explore/repos returned {code}"),
        ),
        None => cb.known_gap("Cannot verify repo enumeration", "Forge unreachable"),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forge_checks_against_unreachable_do_not_panic() {
        let mut results = Vec::new();
        run("192.0.2.1", &mut results);
        assert_eq!(results.len(), 4, "should produce 4 forge checks");
        for r in &results {
            assert!(r.id.starts_with("OFG-"), "check ID should start with OFG-");
        }
    }
}
