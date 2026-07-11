use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process::Command;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};

static TLS_CONFIG: LazyLock<Arc<ClientConfig>> = LazyLock::new(|| {
    let root_store: rustls::RootCertStore =
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect();
    Arc::new(
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .expect("TLS protocol versions")
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    )
});

fn resolve_addr(host: &str, port: u16) -> Option<SocketAddr> {
    format!("{host}:{port}").to_socket_addrs().ok()?.next()
}

pub fn send_raw(host: &str, port: u16, data: &[u8], timeout_ms: u64) -> Option<Vec<u8>> {
    let addr = resolve_addr(host, port)?;
    let timeout = Duration::from_millis(timeout_ms);
    let mut stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
    stream
        .set_read_timeout(Some(Duration::from_millis(timeout_ms.min(3000))))
        .ok()?;
    stream.set_write_timeout(Some(timeout)).ok()?;
    if !data.is_empty() {
        stream.write_all(data).ok()?;
    }
    std::thread::sleep(Duration::from_millis(100));
    let mut buf = vec![0u8; 65536];
    let mut total = Vec::new();
    loop {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                total.extend_from_slice(&buf[..n]);
                if total.len() > 65536 {
                    break;
                }
            }
        }
    }
    Some(total)
}

/// Send a newline-delimited JSON-RPC request over raw TCP.
///
/// Most NUCLEUS primals (`BearDog`, skunkBat, Squirrel, `ToadStool`, barraCuda,
/// coralReef, `NestGate`, biomeOS, sweetGrass, petalTongue) use this framing.
/// Only loamSpine uses HTTP POST JSON-RPC.
pub fn send_jsonrpc_newline(
    host: &str,
    port: u16,
    payload: &str,
    timeout_ms: u64,
) -> Option<String> {
    let mut msg = payload.as_bytes().to_vec();
    if !msg.ends_with(b"\n") {
        msg.push(b'\n');
    }
    let resp = send_raw(host, port, &msg, timeout_ms)?;
    let text = String::from_utf8_lossy(&resp).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

pub fn send_jsonrpc(
    host: &str,
    port: u16,
    payload: &str,
    timeout_ms: u64,
) -> Option<(String, String)> {
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
    let (headers, body) = split_http_response(&text);
    let status = headers.lines().next().unwrap_or("").to_string();
    Some((status, body.to_string()))
}

pub fn http_get(
    host: &str,
    port: u16,
    path: &str,
    extra_headers: &str,
    timeout_ms: u64,
) -> Option<(u16, String, String)> {
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\n{extra_headers}Connection: close\r\n\r\n"
    );
    let resp = send_raw(host, port, req.as_bytes(), timeout_ms)?;
    let text = String::from_utf8_lossy(&resp).to_string();
    let (headers, body) = split_http_response(&text);
    let code = headers
        .lines()
        .next()
        .and_then(parse_status_code)
        .unwrap_or(0);
    Some((code, headers.to_string(), body.to_string()))
}

pub fn http_method(
    host: &str,
    port: u16,
    method: &str,
    path: &str,
    timeout_ms: u64,
) -> Option<u16> {
    let req =
        format!("{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    let resp = send_raw(host, port, req.as_bytes(), timeout_ms)?;
    let text = String::from_utf8_lossy(&resp);
    text.lines().next().and_then(parse_status_code)
}

pub fn http_post(
    host: &str,
    port: u16,
    path: &str,
    content_type: &str,
    body: &str,
    extra_headers: &str,
    timeout_ms: u64,
) -> Option<(u16, String)> {
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: {content_type}\r\n\
         Content-Length: {}\r\n{extra_headers}Connection: close\r\n\r\n{body}",
        body.len()
    );
    let resp = send_raw(host, port, req.as_bytes(), timeout_ms)?;
    let text = String::from_utf8_lossy(&resp).to_string();
    let (headers, resp_body) = split_http_response(&text);
    let code = headers
        .lines()
        .next()
        .and_then(parse_status_code)
        .unwrap_or(0);
    Some((code, resp_body.to_string()))
}

fn tls_stream(
    host: &str,
    port: u16,
    timeout_ms: u64,
) -> Option<StreamOwned<ClientConnection, TcpStream>> {
    let server_name = ServerName::try_from(host.to_string()).ok()?;
    let conn = ClientConnection::new(TLS_CONFIG.clone(), server_name).ok()?;
    let timeout = Duration::from_millis(timeout_ms);
    let addr = resolve_addr(host, port)?;
    let sock = TcpStream::connect_timeout(&addr, timeout).ok()?;
    sock.set_read_timeout(Some(Duration::from_millis(timeout_ms.min(5000))))
        .ok()?;
    sock.set_write_timeout(Some(timeout)).ok()?;
    Some(StreamOwned::new(conn, sock))
}

fn tls_read_response(tls: &mut StreamOwned<ClientConnection, TcpStream>) -> Vec<u8> {
    let mut buf = vec![0u8; 65536];
    let mut total = Vec::new();
    loop {
        match tls.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                total.extend_from_slice(&buf[..n]);
                if total.len() > 131_072 {
                    break;
                }
            }
        }
    }
    total
}

pub fn https_get(
    host: &str,
    path: &str,
    extra_headers: &str,
    timeout_ms: u64,
) -> Option<(u16, String, String)> {
    let mut tls = tls_stream(host, 443, timeout_ms)?;
    let req =
        format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n{extra_headers}Connection: close\r\n\r\n");
    tls.write_all(req.as_bytes()).ok()?;
    let raw = tls_read_response(&mut tls);
    let text = String::from_utf8_lossy(&raw).to_string();
    let (headers, body) = split_http_response(&text);
    let code = headers
        .lines()
        .next()
        .and_then(parse_status_code)
        .unwrap_or(0);
    Some((code, headers.to_string(), body.to_string()))
}

pub fn https_method(host: &str, method: &str, path: &str, timeout_ms: u64) -> Option<u16> {
    let mut tls = tls_stream(host, 443, timeout_ms)?;
    let req = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    tls.write_all(req.as_bytes()).ok()?;
    let raw = tls_read_response(&mut tls);
    let text = String::from_utf8_lossy(&raw);
    text.lines().next().and_then(parse_status_code)
}

/// Probe TLS handshake and return version + cipher info.
/// Sends a minimal HTTP HEAD to force handshake completion, then reads metadata.
pub fn tls_probe(host: &str, timeout_ms: u64) -> Option<TlsInfo> {
    let mut tls = tls_stream(host, 443, timeout_ms)?;
    let req = format!("HEAD / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    tls.write_all(req.as_bytes()).ok()?;
    let mut buf = [0u8; 1];
    let _ = tls.read(&mut buf);
    let version = tls.conn.protocol_version().map(|v| format!("{v:?}"));
    let cipher = tls
        .conn
        .negotiated_cipher_suite()
        .map(|c| format!("{:?}", c.suite()));
    Some(TlsInfo { version, cipher })
}

pub struct TlsInfo {
    pub version: Option<String>,
    pub cipher: Option<String>,
}

pub fn sudo_cmd(user: &str, cmd: &str) -> (i32, String) {
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

/// Parse an HTTP status line into a status code.
/// Used internally by `http_get`, `http_method`, `http_post`.
pub fn parse_status_code(status_line: &str) -> Option<u16> {
    status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
}

/// Split an HTTP response into (headers, body).
pub fn split_http_response(raw: &str) -> (&str, &str) {
    let mut parts = raw.splitn(2, "\r\n\r\n");
    let headers = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or("");
    (headers, body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_code_200() {
        assert_eq!(parse_status_code("HTTP/1.1 200 OK"), Some(200));
    }

    #[test]
    fn parse_status_code_404() {
        assert_eq!(parse_status_code("HTTP/1.1 404 Not Found"), Some(404));
    }

    #[test]
    fn parse_status_code_empty() {
        assert_eq!(parse_status_code(""), None);
    }

    #[test]
    fn parse_status_code_garbage() {
        assert_eq!(parse_status_code("garbage"), None);
    }

    #[test]
    fn split_http_response_normal() {
        let raw = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nhello";
        let (headers, body) = split_http_response(raw);
        assert!(headers.contains("200 OK"));
        assert_eq!(body, "hello");
    }

    #[test]
    fn split_http_response_no_body() {
        let raw = "HTTP/1.1 204 No Content\r\n\r\n";
        let (headers, body) = split_http_response(raw);
        assert!(headers.contains("204"));
        assert_eq!(body, "");
    }

    #[test]
    fn split_http_response_no_separator() {
        let raw = "incomplete response";
        let (headers, body) = split_http_response(raw);
        assert_eq!(headers, "incomplete response");
        assert_eq!(body, "");
    }

    #[test]
    fn send_raw_to_unreachable_returns_none() {
        assert!(send_raw("192.0.2.1", 1, b"test", 200).is_none());
    }

    #[test]
    fn send_jsonrpc_to_unreachable_returns_none() {
        assert!(
            send_jsonrpc(
                "192.0.2.1",
                1,
                r#"{"jsonrpc":"2.0","method":"test","id":1}"#,
                200
            )
            .is_none()
        );
    }

    #[test]
    fn http_get_to_unreachable_returns_none() {
        assert!(http_get("192.0.2.1", 1, "/", "", 200).is_none());
    }

    #[test]
    fn send_jsonrpc_newline_to_unreachable_returns_none() {
        assert!(
            send_jsonrpc_newline(
                "192.0.2.1",
                1,
                r#"{"jsonrpc":"2.0","method":"test","id":1}"#,
                200
            )
            .is_none()
        );
    }

    #[test]
    fn http_method_to_unreachable_returns_none() {
        assert!(http_method("192.0.2.1", 1, "DELETE", "/test", 200).is_none());
    }

    #[test]
    fn http_post_to_unreachable_returns_none() {
        assert!(http_post("192.0.2.1", 1, "/", "application/json", "{}", "", 200).is_none());
    }

    #[test]
    fn https_get_to_unreachable_returns_none() {
        assert!(https_get("192.0.2.1", "/", "", 200).is_none());
    }

    #[test]
    fn https_method_to_unreachable_returns_none() {
        assert!(https_method("192.0.2.1", "DELETE", "/", 200).is_none());
    }

    #[test]
    fn tls_probe_to_unreachable_returns_none() {
        assert!(tls_probe("192.0.2.1", 200).is_none());
    }

    #[test]
    fn sudo_cmd_nonexistent_user_returns_error() {
        let (code, output) = sudo_cmd("__nonexistent_user__", "echo test");
        // Either sudo fails or the command fails — we just verify it doesn't panic
        assert!(code != 0 || output.contains("error") || output.contains("test"));
    }
}
