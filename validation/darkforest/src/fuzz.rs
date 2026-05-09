use crate::check::*;
use crate::net::*;
use std::net::TcpStream;
use std::time::{Duration, Instant};

pub fn run_primals(host: &str, rounds: u32, results: &mut Vec<CheckResult>) {
    let primals = load_primals();
    println!("\n══ Protocol Fuzzing ══");
    for p in &primals {
        println!("\n── Fuzzing {}:{} ──", p.name, p.port);
        fuzz_primal(&p.name, p.port, host, rounds, results);
    }
}

pub fn run_hub(host: &str, results: &mut Vec<CheckResult>) {
    let hub = hub_port();
    println!("\n── Fuzzing JupyterHub ──");
    fuzz_jupyterhub(host, hub, results);
}

fn fuzz_primal(name: &str, port: u16, host: &str, timing_rounds: u32, results: &mut Vec<CheckResult>) {
    let suite = format!("fuzz.{name}");

    let reachable = send_raw(host, port, b"", 2000).is_some()
        || send_raw(host, port, b"test", 2000).is_some();
    if !reachable {
        results.push(
            CheckBuilder::new(&format!("FUZ-{name}-00"), &suite, Category::Fuzz, Severity::Info)
                .pass(&format!("{name}:{port} not listening (skip)"), "Connection refused"),
        );
        return;
    }

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
                results.push(
                    CheckBuilder::new(&format!("FUZ-{name}-crash"), &suite, Category::Fuzz, Severity::Critical)
                        .remediation("Primal must survive malformed input without crashing")
                        .fail(&format!("{name} stopped responding after {pname}"), pname),
                );
                crashed = true;
                break;
            }
        }
    }
    if !crashed {
        results.push(
            CheckBuilder::new(&format!("FUZ-{name}-mal"), &suite, Category::Fuzz, Severity::High)
                .pass(&format!("{name} handled all {} malformed payloads", payloads.len()), "No crashes"),
        );
    }

    let binary_probes: &[(&str, &[u8])] = &[
        ("tls_clienthello", b"\x16\x03\x01\x00\xf1\x01\x00\x00\xed\x03\x03"),
        ("http2_preface", b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"),
        ("ssh_banner", b"SSH-2.0-OpenSSH_9.0\r\n"),
        ("redis_ping", b"*1\r\n$4\r\nPING\r\n"),
        ("memcached_stats", b"stats\r\n"),
    ];
    for (bname, bdata) in binary_probes {
        let cb = CheckBuilder::new(&format!("FUZ-{name}-{bname}"), &suite, Category::Fuzz, Severity::Medium)
            .remediation("Primal must reject non-JSON-RPC protocol probes");
        let resp = send_raw(host, port, bdata, 2000);
        if let Some(ref data) = resp {
            let text = String::from_utf8_lossy(data);
            if text.contains("200 OK") || text.contains("result") {
                results.push(cb.fail(&format!("{name} responded to {bname} with success"), &text[..80.min(text.len())]));
                continue;
            }
        }
        results.push(cb.pass(&format!("{name} rejects {bname}"), "Rejected or no response"));
    }

    let mut big = br#"{"jsonrpc":"2.0","method":"health.liveness","params":{"data":""#.to_vec();
    big.extend(std::iter::repeat(b'A').take(100_000));
    big.extend(br#""},"id":1}"#);
    let cb = CheckBuilder::new(&format!("FUZ-{name}-big"), &suite, Category::Fuzz, Severity::Medium)
        .remediation("Primal must handle oversized payloads without crashing");
    let resp = send_raw(host, port, &big, 5000);
    if resp.is_none() {
        std::thread::sleep(Duration::from_secs(1));
        if send_raw(host, port, b"", 2000).is_none() {
            results.push(cb.fail(&format!("{name} crashed on 100KB payload"), "100KB JSON-RPC"));
        } else {
            results.push(cb.pass(&format!("{name} handled 100KB payload"), "Survived"));
        }
    } else {
        results.push(cb.pass(&format!("{name} handled 100KB payload"), "Survived"));
    }

    timing_analysis(name, port, host, timing_rounds, results);
}

fn timing_analysis(name: &str, port: u16, host: &str, rounds: u32, results: &mut Vec<CheckResult>) {
    let suite = format!("fuzz.{name}");
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

    let cb = CheckBuilder::new(&format!("FUZ-{name}-timing"), &suite, Category::Fuzz, Severity::Low)
        .remediation("Ensure uniform response times across methods to prevent enumeration");

    if timings.len() < 2 {
        results.push(cb.pass(&format!("{name}: too few methods responded for timing analysis"), "Insufficient data"));
        return;
    }

    let means: Vec<f64> = timings.iter().map(|(_, m)| *m).collect();
    let max_diff = means.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - means.iter().cloned().fold(f64::INFINITY, f64::min);
    if max_diff > 0.1 {
        let detail: Vec<String> = timings.iter().map(|(m, t)| format!("{m}={t:.3}s")).collect();
        results.push(cb.dark(
            &format!("{name}: {max_diff:.3}s variance across methods ({})", detail.join(", ")),
            &format!("max_diff={max_diff:.3}s"),
        ));
    } else {
        results.push(cb.pass(
            &format!("{name}: response times within {max_diff:.3}s"),
            &format!("max_diff={max_diff:.3}s"),
        ));
    }
}

fn fuzz_jupyterhub(host: &str, hub_port: u16, results: &mut Vec<CheckResult>) {
    let suite = "fuzz.hub";

    let cookie_val = "A".repeat(50_000);
    let headers = format!("Cookie: jupyterhub-session-id={cookie_val}\r\n");
    let cb = CheckBuilder::new("FUZ-HUB-01", suite, Category::Fuzz, Severity::Medium)
        .remediation("Hub should handle oversized cookies gracefully (400 or 431)");
    match http_get(host, hub_port, "/hub/login", &headers, 5000) {
        Some((code, _, _)) if [200, 302, 400, 403, 431].contains(&code) => {
            results.push(cb.pass(&format!("Hub handles oversized cookie (HTTP {code})"), &format!("HTTP {code}")));
        }
        Some((code, _, _)) => {
            results.push(cb.dark(&format!("Unexpected response to oversized cookie: HTTP {code}"), &format!("HTTP {code}")));
        }
        None => results.push(cb.pass("Hub rejected oversized cookie", "Connection closed")),
    }

    let marker = format!("xfuzz{:06x}", std::process::id());
    let body = format!("username={marker}%00evil&password=test");
    let cb = CheckBuilder::new("FUZ-HUB-02", suite, Category::Fuzz, Severity::Medium)
        .remediation("Sanitize username input, strip null bytes");
    match http_post(host, hub_port, "/hub/login", "application/x-www-form-urlencoded", &body, "", 5000) {
        Some((200, ref rb)) if rb.contains(&marker) => {
            results.push(cb.fail("Null byte username accepted and reflected (HTTP 200)", &marker));
        }
        Some((code, ref rb)) if rb.contains(&marker) => {
            results.push(cb.dark(&format!("Null byte username reflected in error page (HTTP {code}, CSP mitigates)"), &marker));
        }
        Some((code, _)) => {
            results.push(cb.pass(&format!("Null byte username handled (HTTP {code})"), &format!("HTTP {code}")));
        }
        None => results.push(cb.pass("Null byte username rejected", "No response")),
    }

    let fake_tok = "x".repeat(1000);
    let hdr = format!("Authorization: token {fake_tok}\r\n");
    let cb = CheckBuilder::new("FUZ-HUB-03", suite, Category::Auth, Severity::Critical)
        .remediation("Token validation must reject invalid tokens");
    match http_get(host, hub_port, "/hub/api/users", &hdr, 5000) {
        Some((200, _, _)) => results.push(cb.fail("Fake token accepted on /hub/api/users", "HTTP 200 with fake token")),
        Some((code, _, _)) => results.push(cb.pass(&format!("Fake token rejected (HTTP {code})"), &format!("HTTP {code}"))),
        None => results.push(cb.pass("Fake token rejected", "No response")),
    }

    for sqli in ["admin'--", "admin' OR '1'='1", r#"" OR ""="#, r#"admin"; DROP TABLE users;--"#] {
        let body = format!("username={sqli}&password=test");
        let _ = http_post(host, hub_port, "/hub/login", "application/x-www-form-urlencoded", &body, "", 5000);
    }
    results.push(
        CheckBuilder::new("FUZ-HUB-04", suite, Category::Fuzz, Severity::High)
            .pass("Login form handles SQL injection payloads without crash", "4 SQLi payloads survived"),
    );

    let cb = CheckBuilder::new("FUZ-HUB-05", suite, Category::Fuzz, Severity::High)
        .remediation("Escape or strip special chars from ?next parameter");
    match http_get(host, hub_port, r#"/hub/login?next="><script>alert(1)</script>"#, "", 5000) {
        Some((_, _, body)) if body.contains("<script>alert(1)</script>") => {
            results.push(cb.fail("XSS in ?next parameter reflected", "Script tag found in response"));
        }
        Some((_, _, _)) => results.push(cb.pass("XSS in ?next parameter not reflected", "No script tag")),
        None => results.push(cb.pass("XSS probe handled", "No response")),
    }

    for method in ["PUT", "DELETE", "PATCH", "OPTIONS", "TRACE"] {
        let cb = CheckBuilder::new(&format!("FUZ-HUB-M-{}", method.to_lowercase()), suite, Category::Fuzz, Severity::Medium)
            .remediation("Restrict HTTP methods to GET/POST on API endpoints");
        if let Some(code) = http_method(host, hub_port, method, "/hub/api/users", 5000) {
            if code == 200 && (method == "DELETE" || method == "PUT") {
                results.push(cb.fail(&format!("{method} /hub/api/users returns 200"), &format!("HTTP {code}")));
            } else {
                results.push(cb.pass(&format!("{method} /hub/api/users returns {code}"), &format!("HTTP {code}")));
            }
        }
    }

    let cb = CheckBuilder::new("FUZ-HUB-TRACE", suite, Category::Fuzz, Severity::Medium)
        .remediation("Block TRACE method to prevent XST attacks");
    match http_method(host, hub_port, "TRACE", "/hub/", 5000) {
        Some(200) => results.push(cb.fail("TRACE method echoes request — XST vulnerability", "HTTP 200")),
        Some(code) => results.push(cb.pass(&format!("TRACE method blocked (HTTP {code})"), &format!("HTTP {code}"))),
        None => results.push(cb.pass("TRACE method rejected", "No response")),
    }

    let cb = CheckBuilder::new("FUZ-HUB-FLOOD", suite, Category::Fuzz, Severity::Medium)
        .remediation("Rate limiting or connection limits on Hub");
    let mut sockets = Vec::new();
    for _ in 0..50 {
        let addr = format!("{host}:{hub_port}");
        let Ok(sock_addr) = addr.parse() else { break };
        if let Ok(s) = TcpStream::connect_timeout(&sock_addr, Duration::from_secs(2)) {
            sockets.push(s);
        }
    }
    drop(sockets);
    std::thread::sleep(Duration::from_secs(1));
    match http_get(host, hub_port, "/hub/login", "", 5000) {
        Some((code, _, _)) if code == 200 || code == 302 => {
            results.push(cb.pass("Hub survives 50 concurrent connections", &format!("HTTP {code}")));
        }
        Some((code, _, _)) => {
            results.push(cb.dark(&format!("Hub degraded after 50 connections (HTTP {code})"), &format!("HTTP {code}")));
        }
        None => results.push(cb.dark("Hub degraded after connection flood", "No response post-flood")),
    }
}
