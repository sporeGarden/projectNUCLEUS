use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Cloudflare API error: {status} — {message}")]
    Cloudflare { status: u16, message: String },
    #[error("invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfResponse<T> {
    pub success: bool,
    pub errors: Vec<CfError>,
    pub result: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CfError {
    pub code: i64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub connections: Vec<TunnelConnection>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TunnelConnection {
    pub colo_name: Option<String>,
    pub is_pending_reconnect: Option<bool>,
    pub origin_ip: Option<String>,
    pub opened_at: Option<String>,
}

fn cf_error_message(errors: &[CfError]) -> String {
    errors
        .iter()
        .map(|e| e.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

pub struct Client {
    http: reqwest::Client,
    account_id: String,
    base_url: String,
}

impl Client {
    pub fn new(api_token: &str, account_id: &str) -> Result<Self, ApiError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_token}"))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            http,
            account_id: account_id.to_string(),
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        })
    }

    pub async fn get_tunnel(&self, tunnel_id: &str) -> Result<TunnelInfo, ApiError> {
        let url = format!(
            "{}/accounts/{}/cfd_tunnel/{}",
            self.base_url, self.account_id, tunnel_id
        );
        let resp = self.http.get(&url).send().await?;
        let status = resp.status().as_u16();
        let body: CfResponse<TunnelInfo> = resp.json().await?;

        if !body.success {
            let msg = cf_error_message(&body.errors);
            return Err(ApiError::Cloudflare {
                status,
                message: msg,
            });
        }

        body.result
            .ok_or_else(|| ApiError::Other("empty result from CF API".into()))
    }

    pub async fn get_tunnel_config(&self, tunnel_id: &str) -> Result<serde_json::Value, ApiError> {
        let url = format!(
            "{}/accounts/{}/cfd_tunnel/{}/configurations",
            self.base_url, self.account_id, tunnel_id
        );
        let resp = self.http.get(&url).send().await?;
        let status = resp.status().as_u16();
        let body: CfResponse<serde_json::Value> = resp.json().await?;

        if !body.success {
            let msg = cf_error_message(&body.errors);
            return Err(ApiError::Cloudflare {
                status,
                message: msg,
            });
        }

        body.result
            .ok_or_else(|| ApiError::Other("empty config result".into()))
    }

    pub async fn put_tunnel_config(
        &self,
        tunnel_id: &str,
        ingress: &[serde_json::Value],
    ) -> Result<(), ApiError> {
        let url = format!(
            "{}/accounts/{}/cfd_tunnel/{}/configurations",
            self.base_url, self.account_id, tunnel_id
        );
        let payload = serde_json::json!({
            "config": {
                "ingress": ingress
            }
        });
        let resp = self.http.put(&url).json(&payload).send().await?;
        let status = resp.status().as_u16();
        let body: CfResponse<serde_json::Value> = resp.json().await?;

        if !body.success {
            let msg = cf_error_message(&body.errors);
            return Err(ApiError::Cloudflare {
                status,
                message: msg,
            });
        }
        Ok(())
    }

    /// List all Access Applications for the account.
    /// Used by operational scripts — not yet wired into a CLI subcommand.
    #[cfg(feature = "access-apps")]
    #[expect(
        dead_code,
        reason = "Wired by operational scripts via JSON output, not CLI subcommand yet"
    )]
    pub async fn list_access_apps(&self) -> Result<Vec<serde_json::Value>, ApiError> {
        let url = format!("{}/accounts/{}/access/apps", self.base_url, self.account_id);
        let resp = self.http.get(&url).send().await?;
        let status = resp.status().as_u16();
        let body: CfResponse<Vec<serde_json::Value>> = resp.json().await?;

        if !body.success {
            let msg = cf_error_message(&body.errors);
            return Err(ApiError::Cloudflare {
                status,
                message: msg,
            });
        }

        Ok(body.result.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cf_error_message_joins_errors() {
        let errors = vec![
            CfError {
                code: 1000,
                message: "auth failed".to_string(),
            },
            CfError {
                code: 2000,
                message: "rate limited".to_string(),
            },
        ];
        assert_eq!(cf_error_message(&errors), "auth failed; rate limited");
    }

    #[test]
    fn cf_error_message_empty() {
        let errors: Vec<CfError> = vec![];
        assert_eq!(cf_error_message(&errors), "");
    }

    #[test]
    fn cf_response_deserialization() {
        let json = r#"{"success":true,"errors":[],"result":{"id":"abc","name":"test","status":"healthy","connections":[]}}"#;
        let resp: CfResponse<TunnelInfo> = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert!(resp.errors.is_empty());
        assert_eq!(resp.result.unwrap().name, "test");
    }

    #[test]
    fn api_error_display() {
        let e = ApiError::Cloudflare {
            status: 403,
            message: "forbidden".to_string(),
        };
        assert!(e.to_string().contains("403"));
        assert!(e.to_string().contains("forbidden"));
    }

    #[test]
    fn tunnel_info_serialization() {
        let info = TunnelInfo {
            id: "abc".to_string(),
            name: "test-tunnel".to_string(),
            status: "active".to_string(),
            connections: vec![],
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test-tunnel"));
    }
}
