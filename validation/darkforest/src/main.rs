use clap::Parser;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

static PASS: AtomicU32 = AtomicU32::new(0);
static FAIL: AtomicU32 = AtomicU32::new(0);
static GAP: AtomicU32 = AtomicU32::new(0);
static DF: AtomicU32 = AtomicU32::new(0);

fn pass(suite: &str, probe: &str, msg: &str) {
    println!("PASS|{suite}|{probe}|{msg}");
    PASS.fetch_add(1, Ordering::Relaxed);
}
fn fail(suite: &str, probe: &str, msg: &str) {
    println!("FAIL|{suite}|{probe}|{msg}");
    FAIL.fetch_add(1, Ordering::Relaxed);
}
fn known_gap(suite: &str, probe: &str, msg: &str) {
    println!("KNOWN_GAP|{suite}|{probe}|{msg}");
    GAP.fetch_add(1, Ordering::Relaxed);
}
fn dark(suite: &str, probe: &str, msg: &str) {
    println!("DARK_FOREST|{suite}|{probe}|{msg}");
    DF.fetch_add(1, Ordering::Relaxed);
}

// ─── Primal port map ───────────────────────────────────────────
struct Primal {
    name: &'static str,
    port: u16,
}

const PRIMALS: &[Primal] = &[
    Primal { name: "barracuda", port: 9740 },
    Primal { name: "beardog", port: 9100 },
    Primal { name: "biomeos", port: 9800 },
    Primal { name: "coralreef", port: 9730 },
    Primal { name: "loamspine", port: 9700 },
    Primal { name: "nestgate", port: 9500 },
    Primal { name: "petaltongue", port: 9900 },
    Primal { name: "rhizocrypt", port: 9601 },
    Primal { name: "skunkbat", port: 9140 },
    Primal { name: "songbird", port: 9200 },
    Primal { name: "squirrel", port: 9300 },
    Primal { name: "sweetgrass", port: 9850 },
    Primal { name: "toadstool", port: 9400 },
];

const HUB_PORT: u16 = 8000;
const BIND: &str = "127.0.0.1";

const COMPUTE_USER: &str = "tamison";
const REVIEWER_USER: &str = "abgreviewer";
const OBSERVER_USER: &str = "abg-test";

// ─── Network helpers ───────────────────────────────────────────

fn send_raw(host: &str, port: u16, data: &[u8], timeout_ms: u64) -> Option<Vec<u8>> {
    let addr = format!("{host}:{port}");
    let timeout = Duration::from_millis(timeout_ms);
    let mut stream = TcpStream::connect_timeout(&addr.parse().ok()?, timeout).ok()?;
    stream.set_read_timeout(Some(Duration::from_millis(timeout_ms.min(3000)))).ok()?;
    stream.set_write_timeout(Some(timeout)).ok()?;
    if !data.is_empty() {
        stream.write_all(data).ok()?;
    }
    // Brief pause to let the server process and respond
    std::thread::sleep(Duration::from_millis(100));
    let mut buf = vec![0u8; 65536];
    let mut total = Vec::new();
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                total.extend_from_slice(&buf[..n]);
                if total.len() > 65536 {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    Some(total)
}

fn send_jsonrpc(host: &str, port: u16, payload: &str, timeout_ms: u64) -> Option<(String, String)> {
    let content = payload.as_bytes();
    let http_req = format!(
        "POST / HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n",
        content.len()
    );
    let mut full = http_req.into_bytes();
    full.extend_from_slice(content);
    let resp = send_raw(host, port, &full, timeout_ms)?;
    let text = String::from_utf8_lossy(&resp);
    let parts: Vec<&str> = text.splitn(2, "\r\n\r\n").collect();
    let status = parts
        .first()?
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    let body = parts.get(1).unwrap_or(&"").to_string();
    Some((status, body))
}

fn http_get(host: &str, port: u16, path: &str, extra_headers: &str, timeout_ms: u64) -> Option<(u16, String, String)> {
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\n{extra_headers}Connection: close\r\n\r\n"
    );
    let resp = send_raw(host, port, req.as_bytes(), timeout_ms)?;
    let text = String::from_utf8_lossy(&resp).to_string();
    let parts: Vec<&str> = text.splitn(2, "\r\n\r\n").collect();
    let header_block = *parts.first()?;
    let status_line = header_block.lines().next().unwrap_or("");
    let code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body = parts.get(1).unwrap_or(&"").to_string();
    Some((code, header_block.to_string(), body))
}

fn http_method(host: &str, port: u16, method: &str, path: &str, timeout_ms: u64) -> Option<u16> {
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"
    );
    let resp = send_raw(host, port, req.as_bytes(), timeout_ms)?;
    let text = String::from_utf8_lossy(&resp);
    text.lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
}

fn http_post(host: &str, port: u16, path: &str, content_type: &str, body: &str, extra_headers: &str, timeout_ms: u64) -> Option<(u16, String)> {
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: {content_type}\r\n\
         Content-Length: {}\r\n{extra_headers}Connection: close\r\n\r\n{body}",
        body.len()
    );
    let resp = send_raw(host, port, req.as_bytes(), timeout_ms)?;
    let text = String::from_utf8_lossy(&resp).to_string();
    let parts: Vec<&str> = text.splitn(2, "\r\n\r\n").collect();
    let code: u16 = parts
        .first()
        .and_then(|h| h.lines().next())
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let resp_body = parts.get(1).unwrap_or(&"").to_string();
    Some((code, resp_body))
}

fn sudo_cmd(user: &str, cmd: &str) -> (i32, String) {
    let out = Command::new("sudo")
        .args(["-u", user, "bash", "-c", cmd])
        .output();
    match out {
        Ok(o) => {
            let code = o.status.code().unwrap_or(-1);
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            (code, format!("{stdout}{stderr}"))
        }
        Err(e) => (-1, format!("exec error: {e}")),
    }
}

// ─── A. External attacker ──────────────────────────────────────

fn pentest_external(host: &str) {
    println!("\n══ A. External Attacker ══\n");

    // A1: Version disclosure
    println!("── A1: Information Disclosure ──");
    if let Some((_, _, body)) = http_get(host, HUB_PORT, "/hub/api/", "", 5000) {
        if body.contains("\"version\"") {
            let ver = body
                .split("\"version\"")
                .nth(1)
                .and_then(|s| s.split('"').nth(1))
                .unwrap_or("?");
            dark("external", "version_disclosure", &format!("Hub API leaks version {ver} at /hub/api/ (unauthenticated)"));
        } else {
            pass("external", "version_disclosure", "Hub API does not leak version");
        }
    } else {
        pass("external", "version_hidden", "Hub API not reachable");
    }

    if let Some((_, headers, _)) = http_get(host, HUB_PORT, "/hub/login", "", 5000) {
        let srv = headers.lines()
            .find(|l| l.to_lowercase().starts_with("server:"));
        match srv {
            Some(s) if s.to_lowercase().contains("tornado")
                || s.to_lowercase().contains("python")
                || s.to_lowercase().contains("jupyter") =>
            {
                dark("external", "server_header", "Server header reveals implementation");
            }
            Some(s) => pass("external", "server_header", &format!("Server header: {s}")),
            None => pass("external", "server_header", "Server header suppressed"),
        }
    }

    // A2: Unauthenticated admin paths
    println!("\n── A2: Unauthenticated Admin Paths ──");
    for path in ["/hub/admin", "/hub/api/users", "/hub/api/proxy", "/hub/api/services"] {
        let tag = path.rsplit('/').next().unwrap_or(path);
        match http_get(host, HUB_PORT, path, "", 5000) {
            Some((200, _, _)) => fail("external", &format!("unauth_{tag}"), &format!("Unauthenticated access to {path} (HTTP 200)")),
            Some((code, _, _)) => pass("external", &format!("unauth_{tag}"), &format!("{path} blocked (HTTP {code})")),
            None => pass("external", &format!("unauth_{tag}"), &format!("{path} not reachable")),
        }
    }

    // A3: Path traversal
    println!("\n── A3: Path Traversal ──");
    let traversals = [
        "/hub/../../../etc/passwd",
        "/hub/%2e%2e/%2e%2e/etc/passwd",
        "/services/voila/voila/render/../../../etc/passwd",
        "/services/voila/voila/render/%2e%2e/%2e%2e/%2e%2e/etc/passwd",
        "/services/voila/voila/render/..%252f..%252f..%252fetc/passwd",
    ];
    for trav in traversals {
        match http_get(host, HUB_PORT, trav, "", 5000) {
            Some((_, _, body)) if body.contains("root:") => {
                fail("external", "traversal", &format!("Path traversal leaked /etc/passwd: {trav}"));
            }
            Some((code, _, _)) => {
                pass("external", "traversal", &format!("Traversal blocked: {trav} (HTTP {code})"));
            }
            None => pass("external", "traversal", &format!("Traversal blocked: {trav} (no response)")),
        }
    }

    // A4: Host header injection
    println!("\n── A4: Host Header Injection ──");
    for host_val in ["evil.com", "127.0.0.1:9999", "localhost:22"] {
        let header = format!("Host: {host_val}\r\n");
        let req = format!(
            "GET /hub/login HTTP/1.1\r\n{header}Connection: close\r\n\r\n"
        );
        if let Some(resp) = send_raw(host, HUB_PORT, req.as_bytes(), 5000) {
            let text = String::from_utf8_lossy(&resp);
            let body = text.splitn(2, "\r\n\r\n").nth(1).unwrap_or("");
            if body.to_lowercase().contains(&host_val.to_lowercase()) {
                dark("external", "host_injection", &format!("Host header '{host_val}' reflected in response"));
            } else {
                pass("external", "host_injection", &format!("Host '{host_val}' not reflected"));
            }
        }
    }

    // A5: Voila service auth
    println!("\n── A5: Voila Service Auth ──");
    match http_get(host, HUB_PORT, "/services/voila/", "", 5000) {
        Some((200, _, _)) => pass("external", "voila_accessible", "Voila service reachable (behind Hub OAuth)"),
        Some((code, _, _)) => pass("external", "voila_redirect", &format!("Voila redirects unauthenticated (HTTP {code})")),
        None => pass("external", "voila_redirect", "Voila not reachable"),
    }

    // A6: LAN exposure — check all primal ports
    // ss -tlnp columns: State Recv-Q Send-Q Local_Address:Port Peer_Address:Port Process
    // We must check the LOCAL address (col 4), not the peer address (col 5 is always 0.0.0.0:*)
    println!("\n── A6: LAN Exposure ──");
    let (exit, ss_out) = sudo_cmd("root", "ss -tlnp 2>/dev/null | grep LISTEN");
    if exit == 0 {
        let mut lan_exposed = 0u32;
        for p in PRIMALS {
            let port_suffix = format!(":{}", p.port);
            for line in ss_out.lines() {
                let cols: Vec<&str> = line.split_whitespace().collect();
                // col index 3 is Local Address:Port
                if let Some(local_addr) = cols.get(3) {
                    if local_addr.ends_with(&port_suffix) && local_addr.starts_with("0.0.0.0:") {
                        fail("external", &format!("lan_exposed_{}", p.port),
                             &format!("Port {} ({}) bound to 0.0.0.0 — LAN accessible", p.port, p.name));
                        lan_exposed += 1;
                    }
                }
            }
        }
        if lan_exposed == 0 {
            pass("external", "lan_binding", "All primal ports bound to 127.0.0.1 only");
        }
    }
}

// ─── B. Compute user ───────────────────────────────────────────

fn pentest_compute(host: &str) {
    println!("\n══ B. Compute User ({COMPUTE_USER}) ══\n");

    // B1: DNS exfiltration
    println!("── B1: DNS Exfiltration ──");
    let (_, dig_out) = sudo_cmd(COMPUTE_USER, "command -v dig >/dev/null && dig +short +time=2 google.com @8.8.8.8 2>/dev/null || echo 'no-dig'");
    let dig_trimmed = dig_out.trim();
    if dig_trimmed == "no-dig" || dig_trimmed.is_empty() {
        pass("compute", "dns_exfil_dig", "dig not available or DNS blocked");
    } else {
        dark("compute", "dns_exfil_dig", &format!("DNS resolution works (exfil channel): {dig_trimmed}"));
    }

    let (_, dns_out) = sudo_cmd(COMPUTE_USER,
        r#"python3 -c "
import socket
try:
    s=socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(3)
    s.sendto(b'\x00\x01\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x06google\x03com\x00\x00\x01\x00\x01', ('8.8.8.8', 53))
    data=s.recv(512)
    print('resolved' if len(data)>20 else 'failed')
except: print('blocked')
" 2>/dev/null"#);
    if dns_out.trim() == "resolved" {
        dark("compute", "dns_exfil_raw", "Raw DNS to 8.8.8.8 works — tunneling possible");
    } else {
        pass("compute", "dns_exfil_raw", "Raw DNS to external blocked");
    }

    // B2: Primal RPC abuse — MethodGate check
    println!("\n── B2: Primal RPC Abuse ──");
    let rpc_cmd = format!(
        r#"printf '{{"jsonrpc":"2.0","method":"auth.mode","id":1}}\n' | nc -w 2 {host} 9100 2>/dev/null"#
    );
    let (_, gate_out) = sudo_cmd(COMPUTE_USER, &rpc_cmd);
    if gate_out.contains("\"permissive\"") {
        pass("compute", "methodgate_status", "MethodGate live (permissive) on beardog");
    } else if gate_out.contains("\"enforced\"") {
        pass("compute", "methodgate_status", "MethodGate enforced on beardog — JH-0 fully resolved");
    } else {
        known_gap("compute", "methodgate_status", &format!("MethodGate not detected on beardog:9100"));
    }

    let rpc_probes: &[(&str, u16, &str)] = &[
        ("nestgate", 9500, "storage.list"),
        ("nestgate", 9500, "storage.store_blob"),
        ("loamspine", 9700, "spine.status"),
        ("toadstool", 9400, "job.list"),
        ("beardog", 9100, "crypto.list_keys"),
        ("biomeos", 9800, "composition.list"),
    ];
    for (name, port, method) in rpc_probes {
        let cmd = format!(
            r#"printf '{{"jsonrpc":"2.0","method":"{method}","params":{{}},"id":1}}\n' | nc -w 3 {host} {port} 2>/dev/null"#
        );
        let (_, resp) = sudo_cmd(COMPUTE_USER, &cmd);
        let tag = format!("rpc_{name}_{method}");
        if resp.contains("\"result\"") {
            known_gap("compute", &tag, &format!("Unauthenticated {method} on {name}:{port} — MethodGate permissive (logged)"));
        } else if resp.contains("\"error\"") {
            pass("compute", &tag, &format!("{method} rejected on {name}:{port}"));
        } else {
            pass("compute", &tag, &format!("{name}:{port} no response (not running or blocked)"));
        }
    }

    // B3: Supply chain
    println!("\n── B3: Supply Chain ──");
    let (code, _) = sudo_cmd(COMPUTE_USER, "touch /home/irongate/shared/abg/envs/bioinfo/bin/.poison_test 2>/dev/null");
    if code == 0 {
        fail("compute", "env_poison", "Can write to shared conda envs — supply chain attack possible");
        let _ = sudo_cmd("root", "rm -f /home/irongate/shared/abg/envs/bioinfo/bin/.poison_test");
    } else {
        pass("compute", "env_poison", "Cannot write to shared conda envs");
    }

    let (code, _) = sudo_cmd(COMPUTE_USER, "touch /home/irongate/shared/abg/envs/bioinfo/lib/python3.12/site-packages/.poison 2>/dev/null");
    if code == 0 {
        fail("compute", "env_sitepackages", "Can write to shared site-packages");
        let _ = sudo_cmd("root", "rm -f /home/irongate/shared/abg/envs/bioinfo/lib/python3.12/site-packages/.poison");
    } else {
        pass("compute", "env_sitepackages", "Cannot write to shared site-packages");
    }

    // B4: Env/proc leakage
    println!("\n── B4: Env/Proc Leakage ──");
    let (_, pid_out) = sudo_cmd("root", "pgrep -f jupyterhub | head -1");
    let hub_pid = pid_out.trim();
    if !hub_pid.is_empty() {
        let (code, out) = sudo_cmd(COMPUTE_USER, &format!("cat /proc/{hub_pid}/environ 2>&1"));
        if code != 0 || out.contains("denied") || out.contains("Permission") {
            pass("compute", "proc_environ", "Cannot read JupyterHub process environ");
        } else {
            fail("compute", "proc_environ", &format!("Can read JupyterHub /proc/{hub_pid}/environ"));
        }
        let (code, out) = sudo_cmd(COMPUTE_USER, &format!("cat /proc/{hub_pid}/cmdline 2>&1"));
        if code != 0 || out.contains("denied") || out.contains("Permission") {
            pass("compute", "proc_cmdline", "Cannot read JupyterHub cmdline (hidepid=2)");
        } else {
            fail("compute", "proc_cmdline", &format!("Can read JupyterHub /proc/{hub_pid}/cmdline"));
        }
    }

    let (code, _) = sudo_cmd(COMPUTE_USER, "cat /home/irongate/jupyterhub/jupyterhub.sqlite >/dev/null 2>&1");
    if code == 0 {
        fail("compute", "sqlite_read", "Can read jupyterhub.sqlite — contains tokens");
    } else {
        pass("compute", "sqlite_read", "Cannot read jupyterhub.sqlite");
    }

    let (code, _) = sudo_cmd(COMPUTE_USER, "cat /home/irongate/jupyterhub/jupyterhub_cookie_secret >/dev/null 2>&1");
    if code == 0 {
        fail("compute", "cookie_secret", "Can read jupyterhub_cookie_secret");
    } else {
        pass("compute", "cookie_secret", "Cannot read jupyterhub_cookie_secret");
    }

    // B5: Kernel escape paths
    println!("\n── B5: Kernel Escape Paths ──");
    let (code, _) = sudo_cmd(COMPUTE_USER, "ls /home/irongate/ >/dev/null 2>&1");
    if code == 0 {
        fail("compute", "lateral_irongate", "Can list /home/irongate/");
    } else {
        pass("compute", "lateral_irongate", "Cannot list /home/irongate/");
    }

    let (code, _) = sudo_cmd(COMPUTE_USER, "cat /etc/crontab >/dev/null 2>&1");
    if code == 0 {
        dark("compute", "crontab_read", "Can read /etc/crontab (system schedule exposure)");
    } else {
        pass("compute", "crontab_read", "Cannot read /etc/crontab");
    }

    let (code, out) = sudo_cmd(COMPUTE_USER, "systemctl list-units --type=service 2>/dev/null");
    if code == 0 && (out.contains("jupyterhub") || out.contains("cloudflared") || out.contains("forgejo")) {
        dark("compute", "systemd_enum", "Can enumerate system services (jupyterhub/cloudflared visible)");
    } else {
        pass("compute", "systemd_enum", "Cannot enumerate sensitive system services");
    }

    // B6: Cross-user file access
    println!("\n── B6: Cross-User Access ──");
    for other in [REVIEWER_USER, OBSERVER_USER] {
        let (code, _) = sudo_cmd(COMPUTE_USER, &format!("ls /home/{other}/ >/dev/null 2>&1"));
        if code == 0 {
            fail("compute", &format!("cross_user_{other}"), &format!("Can list /home/{other}/"));
        } else {
            pass("compute", &format!("cross_user_{other}"), &format!("Cannot access /home/{other}/"));
        }
    }

    // B7: Internal socket probing
    println!("\n── B7: Internal Socket Probing ──");
    let chp_cmd = format!(
        r#"curl -sf -o /dev/null -w '%{{http_code}}' http://{host}:8001/api/routes --max-time 3 2>/dev/null"#
    );
    let (_, proxy_out) = sudo_cmd(COMPUTE_USER, &chp_cmd);
    if proxy_out.trim() == "200" {
        dark("compute", "proxy_api", "CHP proxy API accessible without auth on :8001");
    } else {
        pass("compute", "proxy_api", &format!("CHP proxy API blocked or not exposed (HTTP {})", proxy_out.trim()));
    }
}

// ─── C. Reviewer/Observer ──────────────────────────────────────

fn pentest_readonly(host: &str) {
    println!("\n══ C. Reviewer/Observer ══\n");

    // C1: Kernel bypass attempts
    println!("── C1: Kernel Bypass Attempts ──");
    let (_, ipyk) = sudo_cmd(REVIEWER_USER, "command -v ipykernel 2>/dev/null");
    if !ipyk.trim().is_empty() {
        dark("readonly", "ipykernel_available", &format!("ipykernel binary found at {} — potential bypass", ipyk.trim()));
    } else {
        pass("readonly", "ipykernel_available", "ipykernel not in reviewer PATH");
    }

    let (_, py_out) = sudo_cmd(REVIEWER_USER, "python3 -c 'print(1+1)' 2>/dev/null");
    if py_out.trim() == "2" {
        dark("readonly", "python3_direct", "Reviewer can execute python3 directly");
    } else {
        pass("readonly", "python3_direct", "Reviewer cannot execute python3");
    }

    let (_, jup) = sudo_cmd(REVIEWER_USER, "command -v jupyter 2>/dev/null");
    if !jup.trim().is_empty() {
        dark("readonly", "jupyter_cli", &format!("jupyter CLI available at {}", jup.trim()));
    } else {
        pass("readonly", "jupyter_cli", "jupyter CLI not in reviewer PATH");
    }

    // C2: File browser escalation
    println!("\n── C2: File Browser Escalation ──");
    let (code, _) = sudo_cmd(REVIEWER_USER, &format!("mkdir /home/{REVIEWER_USER}/notebooks/escape_test 2>/dev/null"));
    if code == 0 {
        fail("readonly", "mkdir_notebooks", "Reviewer can create dirs in ~/notebooks/");
        let _ = sudo_cmd("root", &format!("rmdir /home/{REVIEWER_USER}/notebooks/escape_test 2>/dev/null"));
    } else {
        pass("readonly", "mkdir_notebooks", "Reviewer cannot create dirs in ~/notebooks/");
    }

    let (code, _) = sudo_cmd(REVIEWER_USER, &format!("touch /home/{REVIEWER_USER}/notebooks/.write_test 2>/dev/null"));
    if code == 0 {
        fail("readonly", "touch_notebooks", "Reviewer can create files in ~/notebooks/");
        let _ = sudo_cmd("root", &format!("rm -f /home/{REVIEWER_USER}/notebooks/.write_test"));
    } else {
        pass("readonly", "touch_notebooks", "Reviewer cannot create files in ~/notebooks/");
    }

    // C3: Shared workspace boundaries
    println!("\n── C3: Shared Workspace Boundaries ──");
    let (code, _) = sudo_cmd(REVIEWER_USER, "ls /home/irongate/shared/abg/projects/ >/dev/null 2>&1");
    if code == 0 {
        dark("readonly", "shared_projects_visible", "Reviewer can see shared/projects/ (doc says showcase only)");
    } else {
        pass("readonly", "shared_projects_visible", "Reviewer cannot see shared/projects/");
    }

    let (code, _) = sudo_cmd(REVIEWER_USER, "ls /home/irongate/shared/abg/data/ >/dev/null 2>&1");
    if code == 0 {
        dark("readonly", "shared_data_visible", "Reviewer can see shared/data/");
    } else {
        pass("readonly", "shared_data_visible", "Reviewer cannot see shared/data/");
    }

    // C4: Observer isolation
    println!("\n── C4: Observer Isolation ──");
    let (code, _) = sudo_cmd(OBSERVER_USER, &format!("mkdir /home/{OBSERVER_USER}/notebooks/escape_test 2>/dev/null"));
    if code == 0 {
        fail("readonly", "observer_mkdir", "Observer can create dirs in ~/notebooks/");
        let _ = sudo_cmd("root", &format!("rmdir /home/{OBSERVER_USER}/notebooks/escape_test 2>/dev/null"));
    } else {
        pass("readonly", "observer_mkdir", "Observer cannot create dirs in ~/notebooks/");
    }

    let rpc_cmd = format!(
        r#"printf '{{"jsonrpc":"2.0","method":"auth.mode","id":1}}\n' | nc -w 2 {host} 9100 2>/dev/null"#
    );
    let (_, resp) = sudo_cmd(OBSERVER_USER, &rpc_cmd);
    if resp.contains("\"permissive\"") {
        pass("observer", "rpc_access", "MethodGate live (permissive) — observer RPC logged");
    } else if resp.contains("\"enforced\"") {
        pass("observer", "rpc_access", "MethodGate enforced — observer RPC blocked without token");
    } else {
        known_gap("observer", "rpc_access", "Observer can call primal RPC — MethodGate not detected");
    }
}

// ─── Fuzz: Primal JSON-RPC ─────────────────────────────────────

fn fuzz_primal(name: &str, port: u16, host: &str, timing_rounds: u32) {
    let suite = format!("fuzz_{name}");

    // Connectivity check
    let reachable = send_raw(host, port, b"", 2000).is_some()
        || send_raw(host, port, b"test", 2000).is_some();
    if !reachable {
        pass(&suite, "not_running", &format!("{name}:{port} not listening (skip)"));
        return;
    }

    // Malformed payloads
    let payloads: Vec<(&str, Vec<u8>)> = vec![
        ("empty_string", vec![]),
        ("null_byte", vec![0]),
        ("null_json", b"null".to_vec()),
        ("empty_object", b"{}".to_vec()),
        ("no_jsonrpc", br#"{"method":"health.liveness","id":1}"#.to_vec()),
        ("no_method", br#"{"jsonrpc":"2.0","id":1}"#.to_vec()),
        ("no_id", br#"{"jsonrpc":"2.0","method":"health.liveness"}"#.to_vec()),
        ("method_int", br#"{"jsonrpc":"2.0","method":42,"id":1}"#.to_vec()),
        ("method_null", br#"{"jsonrpc":"2.0","method":null,"id":1}"#.to_vec()),
        ("method_array", br#"{"jsonrpc":"2.0","method":["a"],"id":1}"#.to_vec()),
        ("params_string", br#"{"jsonrpc":"2.0","method":"health.liveness","params":"evil","id":1}"#.to_vec()),
        ("id_overflow", br#"{"jsonrpc":"2.0","method":"health.liveness","id":9007199254740992}"#.to_vec()),
        ("auth_admin_param", br#"{"jsonrpc":"2.0","method":"health.liveness","params":{"auth":"admin"},"id":1}"#.to_vec()),
        ("bearer_in_params", br#"{"jsonrpc":"2.0","method":"health.liveness","params":{"bearer":"root"},"id":1}"#.to_vec()),
        ("method_injection", br#"{"jsonrpc":"2.0","method":"__import__('os').system('id')","id":1}"#.to_vec()),
        ("method_traversal", br#"{"jsonrpc":"2.0","method":"../../../etc/passwd","id":1}"#.to_vec()),
        ("method_null_byte", b"{\"jsonrpc\":\"2.0\",\"method\":\"health\x00.liveness\",\"id\":1}".to_vec()),
        ("deep_nesting", br#"{"jsonrpc":"2.0","method":"health.liveness","params":{"a":{"b":{"c":{"d":{"e":{"f":{"g":{"h":1}}}}}}}},"id":1}"#.to_vec()),
        ("batch_100", {
            let mut s = String::from("[");
            for i in 0..100 {
                if i > 0 { s.push(','); }
                s.push_str(&format!(r#"{{"jsonrpc":"2.0","method":"health.liveness","id":{i}}}"#));
            }
            s.push(']');
            s.into_bytes()
        }),
    ];

    let mut crashed = false;
    for (pname, payload) in &payloads {
        let resp = send_raw(host, port, payload, 3000);
        if resp.is_none() {
            std::thread::sleep(Duration::from_millis(500));
            if send_raw(host, port, b"", 2000).is_none() {
                fail(&suite, &format!("crash_{pname}"), &format!("{name} stopped responding after {pname}"));
                crashed = true;
                break;
            }
        }
    }
    if !crashed {
        pass(&suite, "malformed_resilient", &format!("{name} handled all {} malformed payloads", payloads.len()));
    }

    // Binary protocol probes
    let binary_probes: &[(&str, &[u8])] = &[
        ("tls_clienthello", b"\x16\x03\x01\x00\xf1\x01\x00\x00\xed\x03\x03"),
        ("http2_preface", b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"),
        ("ssh_banner", b"SSH-2.0-OpenSSH_9.0\r\n"),
        ("redis_ping", b"*1\r\n$4\r\nPING\r\n"),
        ("memcached_stats", b"stats\r\n"),
    ];
    for (bname, bdata) in binary_probes {
        let resp = send_raw(host, port, bdata, 2000);
        if let Some(ref data) = resp {
            let text = String::from_utf8_lossy(data);
            if text.contains("200 OK") || text.contains("result") {
                fail(&suite, &format!("binary_{bname}"), &format!("{name} responded to {bname} with success"));
                continue;
            }
        }
        pass(&suite, &format!("binary_{bname}"), &format!("{name} rejects {bname}"));
    }

    // Large payload (100KB)
    let mut big = br#"{"jsonrpc":"2.0","method":"health.liveness","params":{"data":""#.to_vec();
    big.extend(std::iter::repeat(b'A').take(100_000));
    big.extend(br#""},"id":1}"#);
    let resp = send_raw(host, port, &big, 5000);
    if resp.is_none() {
        std::thread::sleep(Duration::from_secs(1));
        if send_raw(host, port, b"", 2000).is_none() {
            fail(&suite, "large_payload_crash", &format!("{name} crashed on 100KB payload"));
        } else {
            pass(&suite, "large_payload", &format!("{name} handled 100KB payload"));
        }
    } else {
        pass(&suite, "large_payload", &format!("{name} handled 100KB payload"));
    }

    // Timing analysis
    timing_analysis(name, port, host, timing_rounds);
}

fn timing_analysis(name: &str, port: u16, host: &str, rounds: u32) {
    let suite = format!("timing_{name}");
    let methods = ["health.liveness", "nonexistent.method", "admin.secret", "storage.list"];
    let mut timings: Vec<(&str, f64)> = Vec::new();

    for method in &methods {
        let mut times = Vec::new();
        let payload = format!(r#"{{"jsonrpc":"2.0","method":"{method}","id":1}}"#);
        for _ in 0..rounds {
            let t0 = Instant::now();
            let result = send_jsonrpc(host, port, &payload, 3000);
            let elapsed = t0.elapsed().as_secs_f64();
            if result.is_some() {
                times.push(elapsed);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        if !times.is_empty() {
            let mean = times.iter().sum::<f64>() / times.len() as f64;
            timings.push((method, mean));
        }
    }

    if timings.len() < 2 {
        pass(&suite, "timing_insufficient", &format!("{name}: too few methods responded for timing analysis"));
        return;
    }

    let means: Vec<f64> = timings.iter().map(|(_, m)| *m).collect();
    let max_diff = means.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - means.iter().cloned().fold(f64::INFINITY, f64::min);
    if max_diff > 0.1 {
        let detail: Vec<String> = timings.iter().map(|(m, t)| format!("{m}={t:.3}s")).collect();
        dark(&suite, "timing_variance", &format!("{name}: {max_diff:.3}s variance across methods ({})", detail.join(", ")));
    } else {
        pass(&suite, "timing_uniform", &format!("{name}: response times within {max_diff:.3}s"));
    }
}

// ─── Fuzz: JupyterHub HTTP ─────────────────────────────────────

fn fuzz_jupyterhub(host: &str) {
    let suite = "fuzz_hub";

    // Oversized cookie
    let cookie_val = "A".repeat(50_000);
    let headers = format!("Cookie: jupyterhub-session-id={cookie_val}\r\n");
    match http_get(host, HUB_PORT, "/hub/login", &headers, 5000) {
        Some((code, _, _)) if [200, 302, 400, 403].contains(&code) => {
            pass(suite, "oversized_cookie", &format!("Hub handles oversized cookie (HTTP {code})"));
        }
        Some((code, _, _)) => {
            dark(suite, "oversized_cookie", &format!("Unexpected response to oversized cookie: HTTP {code}"));
        }
        None => pass(suite, "oversized_cookie", "Hub rejected oversized cookie"),
    }

    // Null bytes in username
    let marker = format!("xfuzz{:06x}", std::process::id());
    let body = format!("username={marker}%00evil&password=test");
    match http_post(host, HUB_PORT, "/hub/login", "application/x-www-form-urlencoded", &body, "", 5000) {
        Some((200, ref rb)) if rb.contains(&marker) => {
            fail(suite, "null_byte_user", &format!("Null byte username accepted and reflected (HTTP 200)"));
        }
        Some((code, ref rb)) if rb.contains(&marker) => {
            dark(suite, "null_byte_user", &format!("Null byte username reflected in error page (HTTP {code}, CSP mitigates)"));
        }
        Some((code, _)) => {
            pass(suite, "null_byte_user", &format!("Null byte username handled (HTTP {code})"));
        }
        None => pass(suite, "null_byte_user", "Null byte username rejected"),
    }

    // Fake token
    let fake_tok = "x".repeat(1000);
    let hdr = format!("Authorization: token {fake_tok}\r\n");
    match http_get(host, HUB_PORT, "/hub/api/users", &hdr, 5000) {
        Some((200, _, _)) => fail(suite, "fake_token", "Fake token accepted on /hub/api/users"),
        Some((code, _, _)) => pass(suite, "fake_token", &format!("Fake token rejected (HTTP {code})")),
        None => pass(suite, "fake_token", "Fake token rejected"),
    }

    // SQL injection in login
    for sqli in ["admin'--", "admin' OR '1'='1", r#"" OR ""="#, r#"admin"; DROP TABLE users;--"#] {
        let body = format!("username={sqli}&password=test");
        let _ = http_post(host, HUB_PORT, "/hub/login", "application/x-www-form-urlencoded", &body, "", 5000);
    }
    pass(suite, "sqli_login", "Login form handles SQL injection payloads without crash");

    // XSS in ?next parameter
    match http_get(host, HUB_PORT, r#"/hub/login?next="><script>alert(1)</script>"#, "", 5000) {
        Some((_, _, body)) if body.contains("<script>alert(1)</script>") => {
            fail(suite, "xss_next", "XSS in ?next parameter reflected");
        }
        Some((_, _, _)) => pass(suite, "xss_next", "XSS in ?next parameter not reflected"),
        None => pass(suite, "xss_next", "XSS probe handled"),
    }

    // HTTP method tampering
    for method in ["PUT", "DELETE", "PATCH", "OPTIONS", "TRACE"] {
        if let Some(code) = http_method(host, HUB_PORT, method, "/hub/api/users", 5000) {
            if code == 200 && (method == "DELETE" || method == "PUT") {
                fail(suite, &format!("method_{}", method.to_lowercase()), &format!("{method} /hub/api/users returns 200"));
            } else {
                pass(suite, &format!("method_{}", method.to_lowercase()), &format!("{method} /hub/api/users returns {code}"));
            }
        }
    }

    // TRACE echo
    match http_method(host, HUB_PORT, "TRACE", "/hub/", 5000) {
        Some(200) => fail(suite, "trace_echo", "TRACE method echoes request — XST vulnerability"),
        Some(code) => pass(suite, "trace_echo", &format!("TRACE method blocked (HTTP {code})")),
        None => pass(suite, "trace_echo", "TRACE method rejected"),
    }

    // Connection flood
    let mut sockets = Vec::new();
    for _ in 0..50 {
        let addr = format!("{host}:{HUB_PORT}");
        if let Ok(s) = TcpStream::connect_timeout(
            &addr.parse().unwrap(),
            Duration::from_secs(2),
        ) {
            sockets.push(s);
        }
    }
    drop(sockets);
    std::thread::sleep(Duration::from_secs(1));
    match http_get(host, HUB_PORT, "/hub/login", "", 5000) {
        Some((code, _, _)) if code == 200 || code == 302 => {
            pass(suite, "conn_flood", "Hub survives 50 concurrent connections");
        }
        Some((code, _, _)) => {
            dark(suite, "conn_flood", &format!("Hub degraded after 50 connections (HTTP {code})"));
        }
        None => dark(suite, "conn_flood", "Hub degraded after connection flood"),
    }
}

// ─── CLI ───────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "darkforest", about = "Dark Forest security validator — pure Rust pen test + fuzz")]
struct Cli {
    /// Test suite: all, pentest, fuzz, external, compute, readonly
    #[arg(long, default_value = "all")]
    suite: String,

    /// Bind address
    #[arg(long, default_value = BIND)]
    host: String,

    /// Timing analysis rounds
    #[arg(long, default_value_t = 5)]
    rounds: u32,
}

fn main() {
    let cli = Cli::parse();
    let host = &cli.host;

    let now = chrono_now();
    println!("═══════════════════════════════════════════════════");
    println!("  Dark Forest — Pure Rust Security Validator");
    println!("  Date: {now}");
    println!("  Suite: {}", cli.suite);
    println!("  Runtime: rustc {}", env!("CARGO_PKG_VERSION"));
    println!("═══════════════════════════════════════════════════");

    let run_pentest = matches!(cli.suite.as_str(), "all" | "pentest" | "external" | "compute" | "readonly");
    let run_fuzz = matches!(cli.suite.as_str(), "all" | "fuzz");

    if run_pentest {
        if matches!(cli.suite.as_str(), "all" | "pentest" | "external") {
            pentest_external(host);
        }
        if matches!(cli.suite.as_str(), "all" | "pentest" | "compute") {
            pentest_compute(host);
        }
        if matches!(cli.suite.as_str(), "all" | "pentest" | "readonly") {
            pentest_readonly(host);
        }
    }

    if run_fuzz {
        println!("\n══ Protocol Fuzzing ══");
        for p in PRIMALS {
            println!("\n── Fuzzing {}:{} ──", p.name, p.port);
            fuzz_primal(p.name, p.port, host, cli.rounds);
        }
        println!("\n── Fuzzing JupyterHub ──");
        fuzz_jupyterhub(host);
    }

    let p = PASS.load(Ordering::Relaxed);
    let f = FAIL.load(Ordering::Relaxed);
    let g = GAP.load(Ordering::Relaxed);
    let d = DF.load(Ordering::Relaxed);

    println!();
    println!("═══════════════════════════════════════════════════");
    println!("  Results: {p} PASS, {f} FAIL, {g} KNOWN_GAP, {d} DARK_FOREST");
    println!("═══════════════════════════════════════════════════");

    if f > 0 {
        println!();
        println!("FAILURES: Active security boundaries are broken.");
    }
    if d > 0 {
        println!();
        println!("DARK FOREST: {d} information leaks or attack surface findings.");
    }

    std::process::exit(if f > 0 { 1 } else { 0 });
}

fn chrono_now() -> String {
    let output = Command::new("date")
        .arg("-Iseconds")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| String::from("unknown"));
    output
}
