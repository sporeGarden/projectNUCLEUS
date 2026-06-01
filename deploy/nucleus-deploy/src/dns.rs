use chrono::Local;
use thiserror::Error;
use tokio::process::Command;

use crate::config::NucleusConfig;

#[derive(Debug, Error)]
pub enum DnsError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSH unreachable: {host}")]
    SshUnreachable { host: String },

    #[error("deployment failed at {phase}: {detail}")]
    DeployFailed { phase: &'static str, detail: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DnsMode {
    Deploy,
    Status,
    Test,
}

pub struct DnsArgs {
    pub mode: DnsMode,
    pub dry_run: bool,
    pub vps_ip: Option<String>,
}

const ZONE_DOMAIN: &str = "primals.eco";
const GHPAGES_IPS: &[&str] = &[
    "185.199.108.153",
    "185.199.109.153",
    "185.199.110.153",
    "185.199.111.153",
];

fn log(msg: &str) {
    eprintln!("[knot-dns] {msg}");
}

fn warn(msg: &str) {
    eprintln!("[knot-dns] WARNING: {msg}");
}

pub async fn run(cfg: &NucleusConfig, args: &DnsArgs) -> Result<(), DnsError> {
    let vps_ip = args.vps_ip.clone().unwrap_or_else(|| cfg.vps_ip.clone());
    let vps_user = cfg.vps_user.clone();

    match args.mode {
        DnsMode::Deploy => do_deploy(&vps_user, &vps_ip, args.dry_run).await,
        DnsMode::Status => do_status(&vps_user, &vps_ip).await,
        DnsMode::Test => do_test(&vps_user, &vps_ip).await,
    }
}

fn generate_zone(vps_ip: &str) -> String {
    let serial = Local::now().format("%Y%m%d%H");
    let a_records: String = GHPAGES_IPS
        .iter()
        .map(|ip| format!("@ IN A {ip}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"$ORIGIN {ZONE_DOMAIN}.
$TTL 300

@ IN SOA ns1.{ZONE_DOMAIN}. admin.{ZONE_DOMAIN}. (
    {serial}   ; serial (YYYYMMDDHH)
    3600       ; refresh (1 hour)
    900        ; retry (15 min)
    1209600    ; expire (2 weeks)
    300        ; minimum TTL (5 min)
)

; Nameservers — sovereign on cellMembrane VPS
@ IN NS ns1.{ZONE_DOMAIN}.

; NS glue record
ns1 IN A {vps_ip}

; Apex — GitHub Pages CDN (extracellular, always-on)
{a_records}

; cellMembrane VPS
membrane IN A {vps_ip}

; Lab surface — pappusCast static content, future BTSP relay
lab IN A {vps_ip}

; golgiBody — periplasmic Forgejo (sovereign git forge)
git IN A {vps_ip}

; CAA — only Let's Encrypt may issue certs
@ IN CAA 0 issue "letsencrypt.org"
@ IN CAA 0 issuewild "letsencrypt.org"

; TXT — domain verification and SPF
@ IN TXT "v=spf1 -all"
"#
    )
}

fn generate_knot_conf(vps_ip: &str) -> String {
    format!(
        r#"server:
    rundir: "/run/knot"
    user: knot:knot
    listen: {vps_ip}@53
    identity: "ns1.{ZONE_DOMAIN}"
    version: ""

log:
  - target: syslog
    any: info

database:
    storage: "/var/lib/knot"

policy:
  - id: ecdsap256
    algorithm: ECDSAP256SHA256
    ksk-lifetime: 365d
    zsk-lifetime: 90d
    nsec3: on
    nsec3-iterations: 0

template:
  - id: default
    storage: "/etc/knot/zones"
    file: "%s.zone"
    semantic-checks: on
    zonefile-sync: -1
    zonefile-load: difference-no-serial
    journal-content: all

zone:
  - domain: {ZONE_DOMAIN}
    dnssec-signing: on
    dnssec-policy: ecdsap256
"#
    )
}

async fn do_deploy(vps_user: &str, vps_ip: &str, dry_run: bool) -> Result<(), DnsError> {
    log(&format!(
        "Deploying knot-dns authoritative for {ZONE_DOMAIN}"
    ));
    log(&format!("  VPS: {vps_ip}"));
    log("  Channel: 1 (Signal)");

    if dry_run {
        deploy_dry_run(vps_ip);
        return Ok(());
    }

    if !ssh_check(vps_user, vps_ip).await {
        return Err(DnsError::SshUnreachable {
            host: format!("{vps_user}@{vps_ip}"),
        });
    }

    deploy_install(vps_user, vps_ip).await?;
    deploy_configure(vps_user, vps_ip).await?;
    deploy_start(vps_user, vps_ip).await?;

    log("");
    log("Phase 7: Validation...");
    do_test(vps_user, vps_ip).await?;

    log("");
    log("═══════════════════════════════════════════════");
    log("  Channel 1 (Signal): knot-dns DEPLOYED");
    log(&format!("  Zone: {ZONE_DOMAIN}"));
    log(&format!("  NS:   ns1.{ZONE_DOMAIN} → {vps_ip}"));
    log("  DNSSEC: ECDSAP256SHA256 (auto-signed)");
    log("═══════════════════════════════════════════════");

    Ok(())
}

fn deploy_dry_run(vps_ip: &str) {
    log("[dry-run] Would install knot-dns from Debian repos");
    log("[dry-run] Would generate /etc/knot/knot.conf");
    log(&format!(
        "[dry-run] Would generate /etc/knot/zones/{ZONE_DOMAIN}.zone"
    ));
    log("[dry-run] Would enable DNSSEC (ECDSAP256SHA256)");
    log("[dry-run] Would open UDP/TCP 53 in UFW");
    log("[dry-run] Would start knot.service");
    log("");
    log("Zone file preview:");
    let zone = generate_zone(vps_ip);
    for line in zone.lines().take(10) {
        log(&format!("  {line}"));
    }
    log("  ...");
}

async fn deploy_install(vps_user: &str, vps_ip: &str) -> Result<(), DnsError> {
    log("Phase 1: Installing knot-dns...");
    let install_script = r#"set -euo pipefail
if command -v knotd >/dev/null 2>&1; then
    echo "knot-dns already installed: $(knotd --version 2>&1 | head -1)"
else
    apt-get update -qq
    apt-get install -y -qq knot knot-dnsutils >/dev/null 2>&1
    echo "Installed: $(knotd --version 2>&1 | head -1)"
fi
mkdir -p /etc/knot/zones /var/lib/knot
chown -R knot:knot /var/lib/knot"#;
    let result = ssh_script(vps_user, vps_ip, install_script).await?;
    log(&format!("  {result}"));
    Ok(())
}

async fn deploy_configure(vps_user: &str, vps_ip: &str) -> Result<(), DnsError> {
    log("Phase 2: Writing configuration...");
    let conf = generate_knot_conf(vps_ip);
    ssh_write(vps_user, vps_ip, "/etc/knot/knot.conf", &conf).await?;

    log("Phase 3: Writing zone file...");
    let zone = generate_zone(vps_ip);
    ssh_write(
        vps_user,
        vps_ip,
        &format!("/etc/knot/zones/{ZONE_DOMAIN}.zone"),
        &zone,
    )
    .await?;
    ssh_cmd(vps_user, vps_ip, "chown -R knot:knot /etc/knot/zones").await?;

    log("Phase 4: Validating configuration...");
    let check = ssh_cmd(vps_user, vps_ip, "knotc conf-check 2>&1").await?;
    if !check.is_empty() && check.contains("error") {
        return Err(DnsError::DeployFailed {
            phase: "conf-check",
            detail: check,
        });
    }
    Ok(())
}

async fn deploy_start(vps_user: &str, vps_ip: &str) -> Result<(), DnsError> {
    log("Phase 5: Opening firewall...");
    let _ = ssh_cmd(
        vps_user,
        vps_ip,
        "ufw allow 53/tcp comment 'Channel 1: DNS (knot-dns)'",
    )
    .await;
    let _ = ssh_cmd(
        vps_user,
        vps_ip,
        "ufw allow 53/udp comment 'Channel 1: DNS (knot-dns)'",
    )
    .await;

    log("Phase 6: Starting knot-dns...");
    ssh_cmd(
        vps_user,
        vps_ip,
        "systemctl enable knot && systemctl restart knot",
    )
    .await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let status = ssh_cmd(
        vps_user,
        vps_ip,
        "systemctl is-active knot 2>/dev/null || echo 'failed'",
    )
    .await?;
    if status.trim() != "active" {
        let journal = ssh_cmd(vps_user, vps_ip, "journalctl -u knot --no-pager -n 15 2>&1")
            .await
            .unwrap_or_default();
        warn(&format!("knot-dns failed to start:\n{journal}"));
        return Err(DnsError::DeployFailed {
            phase: "start",
            detail: format!("status: {status}"),
        });
    }
    log("knot-dns ACTIVE");
    Ok(())
}

async fn do_status(vps_user: &str, vps_ip: &str) -> Result<(), DnsError> {
    log("Channel 1 (Signal) status:");
    let active = ssh_cmd(
        vps_user,
        vps_ip,
        "systemctl is-active knot 2>/dev/null || echo 'not-found'",
    )
    .await?;
    log(&format!("  knot.service: {active}"));

    if active.trim() == "active" {
        let version = ssh_cmd(vps_user, vps_ip, "knotd --version 2>&1 | head -1")
            .await
            .unwrap_or_default();
        log(&format!("  Version: {version}"));

        log("");
        log("  Zone status:");
        let zone_status = ssh_cmd(
            vps_user,
            vps_ip,
            &format!("knotc zone-status {ZONE_DOMAIN} 2>&1"),
        )
        .await
        .unwrap_or_default();
        for line in zone_status.lines() {
            log(&format!("    {line}"));
        }

        log("");
        log("  DNSSEC keys:");
        let keys = ssh_cmd(vps_user, vps_ip, &format!("keymgr {ZONE_DOMAIN} list 2>&1"))
            .await
            .unwrap_or_default();
        for line in keys.lines() {
            log(&format!("    {line}"));
        }

        log("");
        log("  Listeners:");
        let listeners = ssh_cmd(vps_user, vps_ip, "ss -ulnp 2>/dev/null | grep ':53 '")
            .await
            .unwrap_or_default();
        for line in listeners.lines() {
            log(&format!("    {line}"));
        }
    }

    Ok(())
}

async fn do_test(vps_user: &str, vps_ip: &str) -> Result<(), DnsError> {
    log(&format!("Testing resolution against {vps_ip}..."));

    let domains = [
        ZONE_DOMAIN.to_string(),
        format!("ns1.{ZONE_DOMAIN}"),
        format!("membrane.{ZONE_DOMAIN}"),
        format!("lab.{ZONE_DOMAIN}"),
        format!("git.{ZONE_DOMAIN}"),
    ];

    let mut pass = 0u32;
    let mut fail = 0u32;

    for domain in &domains {
        let result = ssh_cmd(
            vps_user,
            vps_ip,
            &format!("khost {domain} localhost 2>/dev/null | head -3"),
        )
        .await
        .unwrap_or_default();

        let result = if result.is_empty() {
            Command::new("dig")
                .args(["+short", &format!("@{vps_ip}"), domain, "A"])
                .output()
                .await
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        } else {
            result
        };

        if result.is_empty() {
            log(&format!("  FAIL: {domain} — no response"));
            fail += 1;
        } else {
            let first_line = result.lines().next().unwrap_or("");
            log(&format!("  PASS: {domain} → {first_line}"));
            pass += 1;
        }
    }

    let dnssec_check = Command::new("dig")
        .args(["+dnssec", "+short", ZONE_DOMAIN, "A", &format!("@{vps_ip}")])
        .output()
        .await;

    if let Ok(o) = dnssec_check {
        let out = String::from_utf8_lossy(&o.stdout);
        if out.contains("RRSIG") {
            log("  PASS: DNSSEC signatures present");
            pass += 1;
        } else {
            log("  INFO: DNSSEC — dig not available or signatures not yet signed");
        }
    }

    let axfr_check = Command::new("dig")
        .args(["AXFR", ZONE_DOMAIN, &format!("@{vps_ip}")])
        .output()
        .await;

    if let Ok(o) = axfr_check {
        let out = String::from_utf8_lossy(&o.stdout);
        if out.contains("Transfer failed") || !out.contains(ZONE_DOMAIN) {
            log("  PASS: AXFR zone transfer blocked");
            pass += 1;
        } else {
            log("  WARN: AXFR zone transfer may be open");
        }
    }

    log("");
    log(&format!("  DNS validation: {pass} PASS, {fail} FAIL"));

    Ok(())
}

async fn ssh_check(user: &str, ip: &str) -> bool {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            &format!("{user}@{ip}"),
            "true",
        ])
        .output()
        .await;
    matches!(output, Ok(o) if o.status.success())
}

async fn ssh_cmd(user: &str, ip: &str, cmd: &str) -> Result<String, DnsError> {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            &format!("{user}@{ip}"),
            cmd,
        ])
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn ssh_script(user: &str, ip: &str, script: &str) -> Result<String, DnsError> {
    let mut child = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            &format!("{user}@{ip}"),
            "bash -s",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(script.as_bytes()).await?;
        drop(stdin);
    }

    let output = child.wait_with_output().await?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn ssh_write(user: &str, ip: &str, remote_path: &str, content: &str) -> Result<(), DnsError> {
    let mut child = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            &format!("{user}@{ip}"),
            &format!("cat > {remote_path}"),
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
        return Err(DnsError::DeployFailed {
            phase: "ssh_write",
            detail: format!("failed to write {remote_path}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_contains_required_records() {
        let zone = generate_zone("1.2.3.4");
        assert!(zone.contains("$ORIGIN primals.eco."));
        assert!(zone.contains("ns1 IN A 1.2.3.4"));
        assert!(zone.contains("membrane IN A 1.2.3.4"));
        assert!(zone.contains("lab IN A 1.2.3.4"));
        assert!(zone.contains("git IN A 1.2.3.4"));
        assert!(zone.contains("IN CAA 0 issue \"letsencrypt.org\""));
        for ip in GHPAGES_IPS {
            assert!(zone.contains(ip), "missing GitHub Pages IP: {ip}");
        }
    }

    #[test]
    fn knot_conf_structure() {
        let conf = generate_knot_conf("1.2.3.4");
        assert!(conf.contains("listen: 1.2.3.4@53"));
        assert!(conf.contains("ECDSAP256SHA256"));
        assert!(conf.contains("primals.eco"));
        assert!(conf.contains("dnssec-signing: on"));
    }

    #[test]
    fn zone_has_spf() {
        let zone = generate_zone("1.2.3.4");
        assert!(zone.contains("v=spf1 -all"));
    }
}
