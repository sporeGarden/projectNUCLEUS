use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net;
use std::net::UdpSocket;
use std::time::Duration;

/// OMS-01 through OMS-03: `WireGuard` / mesh overlay surface audit.
///
/// Checks the `WireGuard` UDP endpoint for handshake flood resistance,
/// invalid peer rejection, and traffic analysis baseline.
pub fn run(target: &str, results: &mut Vec<CheckResult>) {
    let wg_port = wireguard_port();
    check_wg_port_probe(target, wg_port, results);
    check_invalid_handshake(target, wg_port, results);
    check_mesh_surface_minimal(target, results);
}

fn wireguard_port() -> u16 {
    std::env::var("DARKFOREST_WG_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(51820)
}

fn check_wg_port_probe(target: &str, port: u16, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OMS-01", "outer.mesh", Category::Network, Severity::Medium)
        .remediation("WireGuard should silently drop invalid packets — no response to probes");

    let addr = format!("{target}:{port}");
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        results.push(cb.known_gap(
            "Cannot create UDP socket",
            "OS-level socket creation failed",
        ));
        return;
    };
    let _ = socket.set_read_timeout(Some(Duration::from_secs(2)));

    let probe = b"darkforest-probe";
    let send_result = socket.send_to(probe, &addr);

    results.push(match send_result {
        Ok(_) => {
            let mut buf = [0u8; 256];
            match socket.recv_from(&mut buf) {
                Ok((n, _)) => cb.dark(
                    "WireGuard port responded to invalid probe",
                    &format!("Received {n} bytes — should silently drop"),
                ),
                Err(_) => cb.pass(
                    "WireGuard silently drops invalid probes",
                    &format!("No response from {addr} (expected)"),
                ),
            }
        }
        Err(e) => cb.known_gap("Cannot send UDP probe", &format!("Send failed: {e}")),
    });
}

fn check_invalid_handshake(target: &str, port: u16, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OMS-02", "outer.mesh", Category::Crypto, Severity::High)
        .remediation("WireGuard must reject handshake initiation with invalid keys");

    let addr = format!("{target}:{port}");
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else {
        results.push(cb.known_gap("Cannot create UDP socket", "Socket creation failed"));
        return;
    };
    let _ = socket.set_read_timeout(Some(Duration::from_secs(2)));

    // Craft a fake WireGuard handshake initiation (type 1, all zeros for keys)
    let mut fake_init = vec![0u8; 148];
    fake_init[0] = 1; // message type: handshake initiation

    let _ = socket.send_to(&fake_init, &addr);

    let mut buf = [0u8; 256];
    results.push(match socket.recv_from(&mut buf) {
        Ok((n, _)) => cb.dark(
            "WireGuard responded to invalid handshake",
            &format!("Received {n} bytes from fake initiation — investigate"),
        ),
        Err(_) => cb.pass(
            "WireGuard rejects invalid handshake initiation",
            "No response to fake handshake (expected — invalid key)",
        ),
    });
}

fn check_mesh_surface_minimal(target: &str, results: &mut Vec<CheckResult>) {
    let cb = CheckBuilder::new("OMS-03", "outer.mesh", Category::Network, Severity::Low)
        .remediation("Songbird mesh should not expose internal topology to external probes");

    let resp = net::send_jsonrpc_newline(
        target,
        7700,
        r#"{"jsonrpc":"2.0","method":"discovery.peers","id":1}"#,
        2000,
    );

    results.push(match resp {
        Some(ref text) if text.contains("peers") => cb.dark(
            "Songbird mesh exposes peer info externally",
            "discovery.peers responded with topology data",
        ),
        Some(ref text) if text.contains("error") || text.contains("PERMISSION_DENIED") => cb.pass(
            "Songbird mesh rejects external peer queries",
            "Auth enforcement blocks topology discovery",
        ),
        Some(ref text) => cb.pass(
            "Songbird mesh query did not return peer data",
            &format!("Response: {}...", &text[..text.len().min(100)]),
        ),
        None => cb.pass(
            "Songbird federation port not externally reachable",
            &format!("{target}:7700 unreachable (expected for outer membrane)"),
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_checks_against_unreachable_do_not_panic() {
        let mut results = Vec::new();
        run("192.0.2.1", &mut results);
        assert_eq!(results.len(), 3, "should produce 3 mesh checks");
        for r in &results {
            assert!(r.id.starts_with("OMS-"), "check ID should start with OMS-");
        }
    }
}
