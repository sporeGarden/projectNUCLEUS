use std::fmt::Write;
use std::time::Duration;

use reqwest::Client;

fn client(timeout_secs: u64) -> reqwest::Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .danger_accept_invalid_certs(false)
        .build()
}

/// GET request, return body as String.
pub async fn get(url: &str, timeout_secs: u64) -> Option<String> {
    let c = client(timeout_secs).ok()?;
    let resp = c.get(url).send().await.ok()?;
    if resp.status().is_success() {
        resp.text().await.ok()
    } else {
        None
    }
}

/// POST JSON-RPC, return body as String.
pub async fn post_json(url: &str, body: &str, timeout_secs: u64) -> Option<String> {
    let c = client(timeout_secs).ok()?;
    let resp = c
        .post(url)
        .header("Content-Type", "application/json")
        .body(body.to_owned())
        .send()
        .await
        .ok()?;
    resp.text().await.ok()
}

/// HEAD/GET returning only the HTTP status code as a string.
pub async fn status_code(url: &str, timeout_secs: u64) -> Option<String> {
    let c = client(timeout_secs).ok()?;
    let resp = c.get(url).send().await.ok()?;
    Some(resp.status().as_u16().to_string())
}

/// GET returning response headers as a single string (for security header checks).
pub async fn response_headers(url: &str, timeout_secs: u64) -> Option<String> {
    let c = client(timeout_secs).ok()?;
    let resp = c.get(url).send().await.ok()?;
    let mut header_str = String::new();
    for (name, value) in resp.headers() {
        if let Ok(val) = value.to_str() {
            let _ = writeln!(header_str, "{name}: {val}");
        }
    }
    Some(header_str)
}

/// GET with TLS info. Returns `(body, tls_version)`.
pub async fn get_tls_info(url: &str, timeout_secs: u64) -> (Option<String>, Option<String>) {
    let Ok(c) = client(timeout_secs) else {
        return (None, None);
    };
    let Ok(resp) = c.get(url).send().await else {
        return (None, None);
    };
    let version = resp.version();
    let tls_info = match version {
        reqwest::Version::HTTP_2 => Some("HTTP/2 (TLS 1.2+)".to_string()),
        reqwest::Version::HTTP_11 => Some("HTTP/1.1".to_string()),
        _ => Some(format!("{version:?}")),
    };
    let body = resp.text().await.ok();
    (body, tls_info)
}
