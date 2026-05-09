use crate::api;
use crate::config::{ConfigError, TunnelConfig};
use serde::Serialize;
use std::net::TcpStream;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Serialize)]
pub struct HealthReport {
    pub tunnel_name: String,
    pub process: ProcessHealth,
    pub connectivity: ConnectivityHealth,
    pub dns: DnsHealth,
    pub config: ConfigHealth,
    pub overall: String,
}

#[derive(Debug, Serialize)]
pub struct ProcessHealth {
    pub running: bool,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ConnectivityHealth {
    pub local_reachable: bool,
    pub latency_ms: Option<u64>,
    pub cf_edge: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DnsHealth {
    pub resolves: bool,
    pub hostname: String,
    pub resolved_ip: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConfigHealth {
    pub valid: bool,
    pub ingress_rules: usize,
    pub credentials_readable: bool,
}

pub async fn run(
    config_path: &Path,
    api_token: Option<&str>,
    json: bool,
) -> Result<(), ConfigError> {
    let config = TunnelConfig::load(config_path)?;

    let process = check_process();
    let connectivity = check_connectivity(&config).await;
    let dns = check_dns(&config);
    let config_health = check_config(&config);

    let overall = if process.running
        && connectivity.local_reachable
        && dns.resolves
        && config_health.valid
    {
        "healthy".to_string()
    } else {
        let mut issues = Vec::new();
        if !process.running {
            issues.push("process down");
        }
        if !connectivity.local_reachable {
            issues.push("local unreachable");
        }
        if !dns.resolves {
            issues.push("DNS failure");
        }
        if !config_health.valid {
            issues.push("config invalid");
        }
        format!("degraded: {}", issues.join(", "))
    };

    let report = HealthReport {
        tunnel_name: config.tunnel.clone(),
        process,
        connectivity,
        dns,
        config: config_health,
        overall: overall.clone(),
    };

    // If we have an API token, also check remote tunnel status
    if let Some(token) = api_token {
        let creds_raw = std::fs::read_to_string(&config.credentials_file).ok();
        if let Some(raw) = creds_raw {
            if let Ok(creds) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(account_id) = creds.get("AccountTag").and_then(|v| v.as_str()) {
                    let client = api::Client::new(token, account_id);
                    match client.get_tunnel(&config.tunnel).await {
                        Ok(info) => {
                            if !json {
                                println!("CF Edge status: {}", info.status);
                                println!(
                                    "Connections: {}",
                                    info.connections
                                        .iter()
                                        .filter_map(|c| c.colo_name.as_deref())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                );
                            }
                        }
                        Err(e) => {
                            if !json {
                                println!("CF API check failed: {e}");
                            }
                        }
                    }
                }
            }
        }
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print_report(&report);
    }

    Ok(())
}

fn check_process() -> ProcessHealth {
    let output = Command::new("pgrep")
        .args(["-f", "cloudflared.*tunnel"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let pid = stdout
                .lines()
                .next()
                .and_then(|l| l.trim().parse::<u32>().ok());

            let uptime = pid.and_then(|p| {
                let stat = Command::new("ps")
                    .args(["-o", "etimes=", "-p", &p.to_string()])
                    .output()
                    .ok()?;
                String::from_utf8_lossy(&stat.stdout)
                    .trim()
                    .parse::<u64>()
                    .ok()
            });

            ProcessHealth {
                running: true,
                pid,
                uptime_seconds: uptime,
            }
        }
        _ => ProcessHealth {
            running: false,
            pid: None,
            uptime_seconds: None,
        },
    }
}

async fn check_connectivity(config: &TunnelConfig) -> ConnectivityHealth {
    // Probe the first service backend from ingress rules
    let first_service = config
        .ingress
        .iter()
        .find(|r| r.hostname.is_some())
        .map(|r| &r.service);

    if let Some(svc) = first_service {
        if let Some(addr) = svc.strip_prefix("http://").or_else(|| svc.strip_prefix("https://")) {
            let start = Instant::now();
            let reachable = TcpStream::connect_timeout(
                &addr.parse().unwrap_or_else(|_| {
                    std::net::SocketAddr::from(([127, 0, 0, 1], 8000))
                }),
                Duration::from_secs(3),
            )
            .is_ok();
            let latency = start.elapsed().as_millis() as u64;

            return ConnectivityHealth {
                local_reachable: reachable,
                latency_ms: Some(latency),
                cf_edge: None,
            };
        }
    }

    ConnectivityHealth {
        local_reachable: false,
        latency_ms: None,
        cf_edge: None,
    }
}

fn check_dns(config: &TunnelConfig) -> DnsHealth {
    let hostname = config
        .ingress
        .iter()
        .find_map(|r| r.hostname.as_deref())
        .unwrap_or("lab.primals.eco");

    // Try getent (most portable), then host, then dig
    let resolvers: Vec<(&str, Vec<&str>)> = vec![
        ("getent", vec!["hosts", hostname]),
        ("host", vec!["-t", "A", hostname]),
        ("dig", vec!["+short", hostname]),
    ];

    for (cmd, args) in &resolvers {
        if let Ok(output) = Command::new(cmd).args(args).output() {
            if output.status.success() {
                let result = String::from_utf8_lossy(&output.stdout);
                let ip = result
                    .split_whitespace()
                    .find(|w| w.contains('.') || w.contains(':'))
                    .map(String::from);
                if ip.is_some() {
                    return DnsHealth {
                        resolves: true,
                        hostname: hostname.to_string(),
                        resolved_ip: ip,
                    };
                }
            }
        }
    }

    DnsHealth {
        resolves: false,
        hostname: hostname.to_string(),
        resolved_ip: None,
    }
}

fn check_config(config: &TunnelConfig) -> ConfigHealth {
    let creds_readable = Path::new(&config.credentials_file).exists()
        && std::fs::metadata(&config.credentials_file)
            .is_ok_and(|m| m.len() > 0);

    ConfigHealth {
        valid: !config.tunnel.is_empty() && !config.ingress.is_empty(),
        ingress_rules: config.ingress.len(),
        credentials_readable: creds_readable,
    }
}

fn print_report(report: &HealthReport) {
    println!("┌─ Tunnel Keeper Health ─────────────────────");
    println!("│ Tunnel: {}", report.tunnel_name);
    println!("│ Overall: {}", report.overall);
    println!("├─ Process");
    println!(
        "│   Running: {}{}",
        if report.process.running { "yes" } else { "no" },
        report
            .process
            .pid
            .map(|p| format!(" (PID {p})"))
            .unwrap_or_default()
    );
    if let Some(up) = report.process.uptime_seconds {
        let hours = up / 3600;
        let mins = (up % 3600) / 60;
        println!("│   Uptime: {hours}h {mins}m");
    }
    println!("├─ Connectivity");
    println!(
        "│   Local backend: {}",
        if report.connectivity.local_reachable {
            "reachable"
        } else {
            "UNREACHABLE"
        }
    );
    if let Some(ms) = report.connectivity.latency_ms {
        println!("│   Latency: {ms}ms");
    }
    println!("├─ DNS");
    println!(
        "│   {} → {}",
        report.dns.hostname,
        report
            .dns
            .resolved_ip
            .as_deref()
            .unwrap_or("UNRESOLVED")
    );
    println!("├─ Config");
    println!(
        "│   Valid: {} ({} rules)",
        if report.config.valid { "yes" } else { "NO" },
        report.config.ingress_rules
    );
    println!(
        "│   Credentials: {}",
        if report.config.credentials_readable {
            "readable"
        } else {
            "MISSING/EMPTY"
        }
    );
    println!("└──────────────────────────────────────────────");
}
