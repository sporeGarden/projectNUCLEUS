use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Command;

use crate::config::NucleusConfig;

#[derive(Debug, Error)]
pub enum ProvisionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSH unreachable: {host}")]
    SshUnreachable { host: String },

    #[error("phase {phase} failed: {detail}")]
    PhaseFailed { phase: &'static str, detail: String },
}

pub struct ProvisionArgs {
    pub target: String,
    pub dry_run: bool,
    pub full: bool,
    pub plasmid_bin: Option<PathBuf>,
}

fn log(msg: &str) {
    eprintln!("[provision] {msg}");
}

fn warn(msg: &str) {
    eprintln!("[provision] WARNING: {msg}");
}

fn info(msg: &str) {
    eprintln!("[provision] {msg}");
}

pub async fn run(cfg: &NucleusConfig, args: &ProvisionArgs) -> Result<(), ProvisionError> {
    log(&format!("Provisioning gate: {}", args.target));
    let mode_str = if args.full {
        "FULL (primary)"
    } else {
        "REPLICA"
    };
    log(&format!("  Mode: {mode_str}"));

    let (_remote_user, remote_home, hostname) = preflight(args).await?;
    install_gate_env(args, cfg, &remote_home, &hostname).await?;
    deploy_plasmid_bin(args, &remote_home).await?;
    configure_songbird(args, cfg).await?;
    install_services(args, cfg, &remote_home).await?;
    verify_services(args).await;
    print_summary(args, &hostname, mode_str);

    Ok(())
}

async fn preflight(args: &ProvisionArgs) -> Result<(String, String, String), ProvisionError> {
    log("Phase 1: Pre-flight checks");

    if !args.dry_run && !ssh_check(&args.target).await {
        return Err(ProvisionError::SshUnreachable {
            host: args.target.clone(),
        });
    }
    log("  SSH connectivity: OK");

    let remote_user = ssh_cmd_or(&args.target, "whoami", "unknown", args.dry_run).await;
    let default_home = format!("/home/{remote_user}");
    let remote_home = ssh_cmd_or(&args.target, "echo $HOME", &default_home, args.dry_run).await;
    log(&format!("  Remote user: {remote_user} ({remote_home})"));

    let hostname = ssh_cmd_or(&args.target, "hostname", "unknown", args.dry_run).await;
    log(&format!("  Remote hostname: {hostname}"));

    Ok((remote_user, remote_home, hostname))
}

async fn install_gate_env(
    args: &ProvisionArgs,
    cfg: &NucleusConfig,
    remote_home: &str,
    hostname: &str,
) -> Result<(), ProvisionError> {
    log("Phase 2: Gate environment");

    let role = if args.full { "primary" } else { "replica" };
    let gate_env = format!(
        "GATE_HOME={remote_home}\nGATE_NAME={hostname}\nGATE_ROLE={role}\nGATE_BIND={bind}\nSONGBIRD_FEDERATION_PORT={songbird}\n",
        bind = cfg.bind_address,
        songbird = cfg.port_for("songbird"),
    );

    if args.dry_run {
        log("  [dry-run] Would install: /etc/projectnucleus/gate.env");
    } else {
        ssh_cmd(&args.target, "sudo mkdir -p /etc/projectnucleus").await?;
        ssh_write(&args.target, "/etc/projectnucleus/gate.env", &gate_env).await?;
    }
    log("  gate.env installed");
    Ok(())
}

async fn deploy_plasmid_bin(args: &ProvisionArgs, remote_home: &str) -> Result<(), ProvisionError> {
    log("Phase 3: plasmidBin deployment");

    let plasmid_dir = format!("{remote_home}/.local/bin");
    if args.dry_run {
        log(&format!("  [dry-run] Would ensure {plasmid_dir}/ exists"));
    } else {
        ssh_cmd(&args.target, &format!("mkdir -p {plasmid_dir}")).await?;
    }

    let Some(ref plasmid_bin) = args.plasmid_bin else {
        log("  No --plasmid-bin specified, skipping binary deployment");
        return Ok(());
    };

    if !plasmid_bin.exists() {
        warn(&format!(
            "plasmidBin path not found: {}",
            plasmid_bin.display()
        ));
        return Ok(());
    }

    let binaries = list_binaries(plasmid_bin);
    if args.dry_run {
        log(&format!(
            "  [dry-run] Would deploy {} binaries from {}",
            binaries.len(),
            plasmid_bin.display()
        ));
        for b in &binaries {
            log(&format!(
                "    {}",
                b.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
    } else {
        for bin_path in &binaries {
            let name = bin_path.file_name().unwrap_or_default().to_string_lossy();
            log(&format!("  Deploying {name}..."));
            let _ = scp_file(bin_path, &args.target, &format!("{plasmid_dir}/{name}")).await;
            ssh_cmd(&args.target, &format!("chmod +x {plasmid_dir}/{name}")).await?;
        }
        log(&format!("  {} binaries deployed", binaries.len()));
    }

    Ok(())
}

async fn configure_songbird(
    args: &ProvisionArgs,
    cfg: &NucleusConfig,
) -> Result<(), ProvisionError> {
    log("Phase 4: Songbird federation");

    let songbird_env = format!(
        "SONGBIRD_FEDERATION_PORT={}\nSONGBIRD_PEERS={}\n",
        cfg.port_for("songbird"),
        std::env::var("SONGBIRD_PEERS").unwrap_or_default(),
    );

    if args.dry_run {
        log("  [dry-run] Would configure Songbird federation");
    } else {
        ssh_write(
            &args.target,
            "/etc/projectnucleus/songbird.env",
            &songbird_env,
        )
        .await?;
    }
    log("  Songbird federation configured");
    Ok(())
}

async fn install_services(
    args: &ProvisionArgs,
    cfg: &NucleusConfig,
    remote_home: &str,
) -> Result<(), ProvisionError> {
    log("Phase 5: systemd services");

    let services = vec![(
        "observer-static",
        observer_service(remote_home, &cfg.bind_address),
    )];

    for (name, unit) in &services {
        let unit_path = format!("/etc/systemd/system/{name}.service");
        if args.dry_run {
            log(&format!("  [dry-run] Would install: {unit_path}"));
        } else {
            ssh_write(&args.target, &unit_path, unit).await?;
        }
    }

    if !args.dry_run {
        ssh_cmd(&args.target, "sudo systemctl daemon-reload").await?;
        for (name, _) in &services {
            ssh_cmd(
                &args.target,
                &format!("sudo systemctl enable --now {name}.service"),
            )
            .await?;
        }
    }
    log(&format!("  {} services configured", services.len()));
    Ok(())
}

async fn verify_services(args: &ProvisionArgs) {
    log("Phase 6: Verification");
    if args.dry_run {
        return;
    }

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let status = ssh_cmd_or(
        &args.target,
        "systemctl is-active observer-static.service",
        "unknown",
        false,
    )
    .await;
    if status == "active" {
        log("  observer-static: ACTIVE");
    } else {
        warn(&format!("observer-static: {status}"));
    }
}

fn print_summary(args: &ProvisionArgs, hostname: &str, mode_str: &str) {
    log("");
    log("Gate provisioned (sovereign mesh):");
    log(&format!("  Target:    {}", args.target));
    log(&format!("  Mode:      {mode_str}"));
    log(&format!("  Hostname:  {hostname}"));
    info("");
    info("  Sovereign gate model (post-primordial):");
    info("    LAN mesh:  Songbird federation via covalent bond");
    info("    Compute:   plasmidBin binaries deployed to ~/.local/bin");
    info("    Services:  systemd units for observer and primals");
    info("");
    info("  The gate will rejoin the mesh automatically on reboot.");

    if args.dry_run {
        log("");
        warn("DRY RUN — no changes were made");
    }
}

fn observer_service(home: &str, bind: &str) -> String {
    let user = std::env::var("USER").unwrap_or_else(|_| "nobody".into());
    format!(
        r"[Unit]
Description=observer-static — gate static surface
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={user}
ExecStart={home}/.local/bin/observer-static --bind {bind}:8866
Restart=always
RestartSec=5
TimeoutStopSec=10
EnvironmentFile=/etc/projectnucleus/gate.env

[Install]
WantedBy=multi-user.target",
    )
}

fn list_binaries(dir: &std::path::Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir).map_or_else(
        |_| Vec::new(),
        |entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| {
                    e.file_type().is_ok_and(|ft| ft.is_file()) && e.path().extension().is_none()
                })
                .map(|e| e.path())
                .collect()
        },
    )
}

async fn ssh_check(target: &str) -> bool {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            target,
            "true",
        ])
        .output()
        .await;
    matches!(output, Ok(o) if o.status.success())
}

async fn ssh_cmd(target: &str, cmd: &str) -> Result<String, ProvisionError> {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            target,
            cmd,
        ])
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn ssh_cmd_or(target: &str, cmd: &str, default: &str, dry_run: bool) -> String {
    if dry_run {
        return default.to_string();
    }
    ssh_cmd(target, cmd)
        .await
        .unwrap_or_else(|_| default.into())
}

async fn ssh_write(target: &str, remote_path: &str, content: &str) -> Result<(), ProvisionError> {
    let mut child = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            target,
            &format!("sudo tee {remote_path} > /dev/null"),
        ])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(content.as_bytes()).await?;
        drop(stdin);
    }

    let status = child.wait().await?;
    if !status.success() {
        return Err(ProvisionError::PhaseFailed {
            phase: "ssh_write",
            detail: format!("failed to write {remote_path}"),
        });
    }
    Ok(())
}

async fn scp_file(
    local: &std::path::Path,
    target: &str,
    remote: &str,
) -> Result<(), ProvisionError> {
    let output = Command::new("scp")
        .args([
            "-q",
            &local.to_string_lossy(),
            &format!("{target}:{remote}"),
        ])
        .output()
        .await?;

    if !output.status.success() {
        return Err(ProvisionError::PhaseFailed {
            phase: "scp",
            detail: format!("failed to copy {} to {target}:{remote}", local.display()),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observer_service_template() {
        let unit = observer_service("/home/test", "127.0.0.1");
        assert!(unit.contains("observer-static"));
        assert!(unit.contains("127.0.0.1:8866"));
        assert!(unit.contains("EnvironmentFile=/etc/projectnucleus/gate.env"));
    }

    #[test]
    fn list_binaries_empty() {
        let result = list_binaries(std::path::Path::new("/nonexistent/path"));
        assert!(result.is_empty());
    }

    #[test]
    fn observer_service_contains_restart_policy() {
        let unit = observer_service("/home/test", "127.0.0.1");
        assert!(unit.contains("Restart=always"));
        assert!(unit.contains("[Install]"));
    }
}
