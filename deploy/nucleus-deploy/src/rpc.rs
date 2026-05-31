use std::time::Duration;

use serde_json::Value;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

const RPC_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("connection refused ({addr})")]
    ConnectionRefused { addr: String },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("timeout after {0:?}")]
    Timeout(Duration),

    #[error("invalid JSON response: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("empty response from {addr}")]
    EmptyResponse { addr: String },
}

pub struct RpcResponse {
    pub raw: String,
    pub parsed: Value,
}

impl RpcResponse {
    pub fn has_result(&self) -> bool {
        self.parsed.get("result").is_some()
    }

    pub fn has_error(&self) -> bool {
        self.parsed.get("error").is_some()
    }

    pub fn result(&self) -> Option<&Value> {
        self.parsed.get("result")
    }
}

/// Send a newline-delimited JSON-RPC request to a TCP endpoint.
pub async fn send_jsonrpc(host: &str, port: u16, payload: &str) -> Result<RpcResponse, RpcError> {
    send_jsonrpc_with_timeout(host, port, payload, RPC_TIMEOUT).await
}

pub async fn send_jsonrpc_with_timeout(
    host: &str,
    port: u16,
    payload: &str,
    dur: Duration,
) -> Result<RpcResponse, RpcError> {
    let addr = format!("{host}:{port}");

    let result = timeout(dur, async {
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|_| RpcError::ConnectionRefused { addr: addr.clone() })?;

        let (reader, mut writer) = stream.into_split();
        let mut line_payload = payload.to_string();
        if !line_payload.ends_with('\n') {
            line_payload.push('\n');
        }
        writer.write_all(line_payload.as_bytes()).await?;
        writer.shutdown().await?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();
        buf_reader.read_line(&mut response).await?;

        if response.trim().is_empty() {
            return Err(RpcError::EmptyResponse { addr });
        }

        let parsed: Value = serde_json::from_str(response.trim())?;
        Ok(RpcResponse {
            raw: response,
            parsed,
        })
    })
    .await;

    result.map_or(Err(RpcError::Timeout(dur)), |inner| inner)
}

/// Send a raw TCP payload (not necessarily JSON-RPC). Returns raw response.
pub async fn send_raw_tcp(
    host: &str,
    port: u16,
    payload: &[u8],
    dur: Duration,
) -> Result<String, RpcError> {
    let addr = format!("{host}:{port}");

    let result = timeout(dur, async {
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|_| RpcError::ConnectionRefused { addr: addr.clone() })?;

        let (reader, mut writer) = stream.into_split();
        writer.write_all(payload).await?;
        writer.shutdown().await?;

        let mut buf_reader = BufReader::new(reader);
        let mut response = String::new();
        buf_reader.read_line(&mut response).await?;
        Ok(response)
    })
    .await;

    result.map_or(Err(RpcError::Timeout(dur)), |inner| inner)
}

/// Build a JSON-RPC 2.0 request string.
pub fn jsonrpc_request(method: &str, id: u64) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": {},
        "id": id,
    })
    .to_string()
}

/// Build a JSON-RPC 2.0 request with params.
pub fn jsonrpc_request_with_params(method: &str, params: &Value, id: u64) -> String {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    })
    .to_string()
}

/// Check if a primal is alive by probing health.liveness.
pub async fn check_liveness(host: &str, port: u16) -> bool {
    let req = jsonrpc_request("health.liveness", 0);
    matches!(send_jsonrpc(host, port, &req).await, Ok(r) if r.has_result())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonrpc_request_is_valid_json() {
        let req = jsonrpc_request("health.liveness", 1);
        let parsed: Value = serde_json::from_str(&req).expect("valid JSON");
        assert_eq!(parsed["method"], "health.liveness");
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
    }

    #[test]
    fn jsonrpc_with_params_includes_params() {
        let params = serde_json::json!({"key": "value"});
        let req = jsonrpc_request_with_params("storage.list", &params, 42);
        let parsed: Value = serde_json::from_str(&req).expect("valid JSON");
        assert_eq!(parsed["params"]["key"], "value");
        assert_eq!(parsed["id"], 42);
    }
}
