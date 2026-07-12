use std::path::Path;

use tokio::fs;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::config::NucleusConfig;

pub struct PrimalContext<'a> {
    pub cfg: &'a NucleusConfig,
    pub bind: &'a str,
    pub family_id: &'a str,
    pub node_id: &'a str,
    pub beacon_seed: &'a Path,
    pub beardog_socket: &'a Path,
    pub uds_only: bool,
}

pub async fn start_primal(ctx: &PrimalContext<'_>, name: &str) {
    let bin = ctx.cfg.plasmidbin_dir.join(format!("primals/{name}"));
    if !bin.exists() {
        eprintln!("  SKIP {name} — no binary found");
        return;
    }

    let port_for = |port: u16| -> u16 {
        if ctx.uds_only {
            0
        } else {
            port
        }
    };

    let mut cmd = Command::new(&bin);
    configure_primal_cmd(&mut cmd, ctx, name, port_for);

    let log_path = format!("/tmp/{name}.log");
    let log_file = std::fs::File::create(&log_path).ok();

    if let Some(f) = log_file {
        if let Ok(clone) = f.try_clone() {
            cmd.stdout(clone);
        }
        cmd.stderr(f);
    }

    let auth_mode = std::env::var("NUCLEUS_AUTH_MODE").unwrap_or_else(|_| "enforced".into());
    cmd.env(format!("{}_AUTH_MODE", name.to_uppercase()), &auth_mode);

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id().unwrap_or(0);
            eprintln!("    PID: {pid}");
        }
        Err(e) => eprintln!("    FAILED: {e}"),
    }

    let delay = match name {
        "beardog" | "songbird" | "biomeos" | "nestgate" => 2,
        _ => 1,
    };
    sleep(Duration::from_secs(delay)).await;
}

#[expect(
    clippy::too_many_lines,
    reason = "per-primal match dispatch — single logical unit"
)]
fn configure_primal_cmd(
    cmd: &mut Command,
    ctx: &PrimalContext<'_>,
    name: &str,
    port_for: impl Fn(u16) -> u16,
) {
    let cfg = ctx.cfg;
    let bind = ctx.bind;
    let family_id = ctx.family_id;
    let runtime = &cfg.runtime_dir;

    match name {
        "beardog" => {
            let port = port_for(cfg.port_for("beardog"));
            let transport = if port > 0 {
                format!("UDS + TCP {port}")
            } else {
                "UDS-only".into()
            };
            eprintln!("  Starting beardog ({transport})...");
            cmd.env("BEARDOG_FAMILY_SEED", ctx.beacon_seed);
            cmd.args(["server", "--socket"]);
            cmd.arg(ctx.beardog_socket);
            cmd.args(["--family-id", family_id]);
            if port > 0 {
                cmd.args(["--listen", &format!("{bind}:{port}")]);
            }
        }
        "songbird" => {
            let port = port_for(cfg.port_for("songbird"));
            let transport = if port > 0 {
                format!("HTTP {port}")
            } else {
                "UDS-only".into()
            };
            eprintln!("  Starting songbird ({transport})...");
            cmd.env("BEARDOG_SOCKET", ctx.beardog_socket);
            cmd.env("BEARDOG_MODE", "direct");
            cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog");
            let sock = runtime.join(format!("biomeos/songbird-{family_id}.sock"));
            cmd.args(["server", "--socket"]);
            cmd.arg(&sock);
            if port > 0 {
                cmd.args(["--port", &port.to_string()]);
            }
        }
        "toadstool" => {
            let port = port_for(cfg.port_for("toadstool"));
            eprintln!("  Starting toadstool (TCP {port})...");
            cmd.env("TOADSTOOL_FAMILY_ID", family_id);
            cmd.env("TOADSTOOL_NODE_ID", ctx.node_id);
            cmd.env("TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED", "1");
            cmd.args(["server", "--family-id", family_id]);
            if port > 0 {
                cmd.args(["--port", &port.to_string()]);
            }
        }
        "nestgate" => {
            let port = port_for(cfg.port_for("nestgate"));
            eprintln!("  Starting nestgate (TCP {port})...");
            cmd.env("NESTGATE_FAMILY_ID", family_id);
            cmd.args(["daemon", "--socket-only"]);
            if port > 0 {
                cmd.args(["--port", &port.to_string(), "--bind", bind]);
            }
        }
        "rhizocrypt" => {
            let port = port_for(cfg.port_for("rhizocrypt"));
            eprintln!("  Starting rhizocrypt (TCP {port})...");
            cmd.env("FAMILY_SEED", ctx.beacon_seed);
            cmd.arg("server");
            if port > 0 {
                cmd.args(["--port", &port.to_string(), "--host", bind]);
            }
        }
        "loamspine" => {
            let port = port_for(cfg.port_for("loamspine"));
            eprintln!("  Starting loamspine (TCP {port})...");
            cmd.arg("server");
            if port > 0 {
                cmd.args(["--port", &port.to_string(), "--bind-address", bind]);
            }
        }
        "sweetgrass" => {
            let port = port_for(cfg.port_for("sweetgrass"));
            eprintln!("  Starting sweetgrass (TCP {port})...");
            cmd.arg("server");
            if port > 0 {
                let http_port = port + 1;
                cmd.args([
                    "--port",
                    &port.to_string(),
                    "--http-address",
                    &format!("{bind}:{http_port}"),
                ]);
            }
        }
        "biomeos" => {
            let port = port_for(cfg.port_for("biomeos"));
            eprintln!("  Starting biomeos (TCP {port})...");
            cmd.arg("neural-api");
            if port > 0 {
                cmd.args(["--port", &port.to_string()]);
            }
        }
        _ => {
            let port = cfg.port_for(name);
            let p = port_for(port);
            eprintln!("  Starting {name} (TCP {p})...");
            cmd.arg("server");
            if p > 0 {
                cmd.args(["--port", &p.to_string()]);
            }
        }
    }
}

pub async fn verify_primals(
    cfg: &NucleusConfig,
    primals: &[&str],
    uds_only: bool,
    runtime_dir: &Path,
) -> bool {
    let mut all_ok = true;

    for &p in primals {
        let pattern = format!("{}/primals/{p}", cfg.plasmidbin_dir.display());
        let output = Command::new("pgrep").args(["-f", &pattern]).output().await;

        let pid = output.ok().filter(|o| o.status.success()).map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        });

        match pid {
            Some(pid) if !pid.is_empty() => {
                if uds_only {
                    let biomeos_dir = runtime_dir.join("biomeos");
                    let has_socket = has_socket_for(&biomeos_dir, p).await;
                    if has_socket {
                        eprintln!("  {p}: PID {pid}, SOCKET LIVE");
                    } else {
                        eprintln!("  {p}: PID {pid}, SOCKET ABSENT — running (socket pending)");
                    }
                } else {
                    let port = cfg.port_for(p);
                    if port > 0 {
                        let healthy = crate::rpc::check_liveness(&cfg.bind_address, port).await;
                        if healthy {
                            eprintln!("  {p}: PID {pid}, TCP {port} — HEALTHY");
                        } else {
                            eprintln!(
                                "  {p}: PID {pid}, TCP {port} — running (health probe pending)"
                            );
                        }
                    } else {
                        eprintln!("  {p}: PID {pid} — running");
                    }
                }
            }
            _ => {
                eprintln!("  {p}: NOT RUNNING — check /tmp/{p}.log");
                all_ok = false;
            }
        }
    }

    all_ok
}

async fn has_socket_for(dir: &Path, primal: &str) -> bool {
    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(primal)
                && Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
            {
                return true;
            }
        }
    }
    false
}

pub async fn hostname() -> String {
    Command::new("hostname")
        .arg("-s")
        .output()
        .await
        .ok()
        .map_or_else(
            || "unknown".into(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )
}
