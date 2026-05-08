use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Command;
use std::time::Duration;

pub fn send_raw(host: &str, port: u16, data: &[u8], timeout_ms: u64) -> Option<Vec<u8>> {
    let addr = format!("{host}:{port}");
    let timeout = Duration::from_millis(timeout_ms);
    let mut stream = TcpStream::connect_timeout(&addr.parse().ok()?, timeout).ok()?;
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
    let parts: Vec<&str> = text.splitn(2, "\r\n\r\n").collect();
    let status = parts.first()?.lines().next().unwrap_or("").to_string();
    let body = parts.get(1).unwrap_or(&"").to_string();
    Some((status, body))
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

pub fn http_method(
    host: &str,
    port: u16,
    method: &str,
    path: &str,
    timeout_ms: u64,
) -> Option<u16> {
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
