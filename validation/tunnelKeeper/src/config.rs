use crate::api;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    NotFound(String),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("API error: {0}")]
    Api(#[from] api::ApiError),
    #[error("crypto error: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig {
    pub tunnel: String,
    #[serde(rename = "credentials-file")]
    pub credentials_file: String,
    pub ingress: Vec<IngressRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub service: String,
}

impl TunnelConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.display().to_string()));
        }
        let raw = fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&raw)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(path, yaml)?;
        Ok(())
    }
}

pub fn show(config_path: &Path, json: bool) -> Result<(), ConfigError> {
    let config = TunnelConfig::load(config_path)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&config).unwrap_or_default()
        );
    } else {
        println!("Tunnel: {}", config.tunnel);
        println!("Credentials: {}", config.credentials_file);
        println!("\nIngress rules ({}):", config.ingress.len());
        for (i, rule) in config.ingress.iter().enumerate() {
            let host = rule.hostname.as_deref().unwrap_or("*");
            let path = rule.path.as_deref().unwrap_or("");
            println!("  [{i}] {host}{path} → {}", rule.service);
        }
    }
    Ok(())
}

pub async fn sync(
    config_path: &Path,
    api_token: Option<&str>,
    pull: bool,
    json: bool,
) -> Result<(), ConfigError> {
    let config = TunnelConfig::load(config_path)?;
    let token = api_token.ok_or_else(|| {
        ConfigError::Other("CF_API_TOKEN required for sync (set --api-token or env var)".into())
    })?;

    let creds_raw = fs::read_to_string(&config.credentials_file)?;
    let creds: serde_json::Value = serde_json::from_str(&creds_raw)
        .map_err(|e| ConfigError::Other(format!("credentials parse error: {e}")))?;
    let account_id = creds
        .get("AccountTag")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::Other("AccountTag not found in credentials".into()))?;

    let client = api::Client::new(token, account_id)
        .map_err(|e| ConfigError::Other(format!("API client init failed: {e}")))?;

    if pull {
        let remote = client.get_tunnel_config(&config.tunnel).await?;
        if !json {
            println!("Remote config for tunnel '{}':", config.tunnel);
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&remote).unwrap_or_default()
        );
    } else {
        let ingress_payload: Vec<serde_json::Value> = config
            .ingress
            .iter()
            .filter_map(|r| serde_json::to_value(r).ok())
            .collect();

        client
            .put_tunnel_config(&config.tunnel, &ingress_payload)
            .await?;

        if json {
            println!(
                r#"{{"status":"synced","tunnel":"{}","rules":{}}}"#,
                config.tunnel,
                config.ingress.len()
            );
        } else {
            println!(
                "Pushed {} ingress rules to tunnel '{}'",
                config.ingress.len(),
                config.tunnel
            );
        }
    }
    Ok(())
}

pub fn route_list(config_path: &Path, json: bool) -> Result<(), ConfigError> {
    let config = TunnelConfig::load(config_path)?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&config.ingress).unwrap_or_default()
        );
    } else {
        println!("Ingress rules ({}):", config.ingress.len());
        for (i, rule) in config.ingress.iter().enumerate() {
            let host = rule.hostname.as_deref().unwrap_or("*");
            let path = rule.path.as_deref().unwrap_or("");
            let path_display = if path.is_empty() {
                String::new()
            } else {
                format!(" path={path}")
            };
            println!("  [{i}] {host}{path_display} → {}", rule.service);
        }
    }
    Ok(())
}

pub fn route_add(
    config_path: &Path,
    hostname: &str,
    path: Option<&str>,
    service: &str,
    json: bool,
) -> Result<(), ConfigError> {
    let mut config = TunnelConfig::load(config_path)?;

    let new_rule = IngressRule {
        hostname: Some(hostname.to_string()),
        path: path.map(String::from),
        service: service.to_string(),
    };

    // Insert before the catch-all rule (last rule)
    let insert_pos = if config.ingress.len() > 1 {
        config.ingress.len() - 1
    } else {
        config.ingress.len()
    };
    config.ingress.insert(insert_pos, new_rule);
    config.save(config_path)?;

    if json {
        println!(
            r#"{{"status":"added","hostname":"{}","path":"{}","service":"{}"}}"#,
            hostname,
            path.unwrap_or(""),
            service
        );
    } else {
        println!("Added rule: {hostname}{} → {service}", path.unwrap_or(""));
        println!(
            "Config written to {}. Restart cloudflared to apply.",
            config_path.display()
        );
    }
    Ok(())
}

pub fn route_rm(config_path: &Path, path: &str, json: bool) -> Result<(), ConfigError> {
    let mut config = TunnelConfig::load(config_path)?;

    let before = config.ingress.len();
    config.ingress.retain(|r| r.path.as_deref() != Some(path));
    let removed = before - config.ingress.len();

    if removed == 0 {
        return Err(ConfigError::Other(format!(
            "no rule with path '{path}' found"
        )));
    }

    config.save(config_path)?;

    if json {
        println!(r#"{{"status":"removed","path":"{path}","count":{removed}}}"#);
    } else {
        println!("Removed {removed} rule(s) with path '{path}'");
        println!(
            "Config written to {}. Restart cloudflared to apply.",
            config_path.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r"
tunnel: abc-123-def
credentials-file: /home/user/.cloudflared/abc-123-def.json
ingress:
  - hostname: lab.primals.eco
    service: http://127.0.0.1:8000
  - hostname: git.primals.eco
    service: http://127.0.0.1:3000
  - service: http_status:404
";

    #[test]
    fn parse_valid_config() {
        let config: TunnelConfig = serde_yaml::from_str(SAMPLE_YAML).unwrap();
        assert_eq!(config.tunnel, "abc-123-def");
        assert_eq!(config.ingress.len(), 3);
        assert_eq!(
            config.ingress[0].hostname.as_deref(),
            Some("lab.primals.eco")
        );
        assert!(config.ingress[2].hostname.is_none());
    }

    #[test]
    fn config_yaml_roundtrip() {
        let config: TunnelConfig = serde_yaml::from_str(SAMPLE_YAML).unwrap();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let config2: TunnelConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config.tunnel, config2.tunnel);
        assert_eq!(config.ingress.len(), config2.ingress.len());
    }

    #[test]
    fn config_json_serialization() {
        let config: TunnelConfig = serde_yaml::from_str(SAMPLE_YAML).unwrap();
        let json = serde_json::to_string(&config).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["tunnel"], "abc-123-def");
    }

    #[test]
    fn load_nonexistent_path_returns_not_found() {
        let result = TunnelConfig::load(Path::new("/nonexistent/path/config.yml"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"), "expected NotFound, got: {err}");
    }

    #[test]
    fn config_save_and_reload() {
        let dir = std::env::temp_dir().join("tunnelkeeper_test_config");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_config.yml");

        let config: TunnelConfig = serde_yaml::from_str(SAMPLE_YAML).unwrap();
        config.save(&path).unwrap();

        let loaded = TunnelConfig::load(&path).unwrap();
        assert_eq!(loaded.tunnel, "abc-123-def");
        assert_eq!(loaded.ingress.len(), 3);

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn ingress_rule_hostname_none_for_catchall() {
        let rule: IngressRule = serde_yaml::from_str("service: http_status:404").unwrap();
        assert!(rule.hostname.is_none());
        assert!(rule.path.is_none());
        assert_eq!(rule.service, "http_status:404");
    }

    #[test]
    fn ingress_rule_with_path() {
        let yaml = "hostname: lab.primals.eco\npath: \"/api/.*\"\nservice: http://127.0.0.1:8000\n";
        let rule: IngressRule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.path.as_deref(), Some("/api/.*"));
    }
}
