use crate::check::*;
use crate::net::sudo_cmd;
use super::{CryptoConfig, shannon_entropy, hex_decode};
use std::time::SystemTime;

pub(crate) fn cry_01_cookie_entropy(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-01: Cookie Secret Entropy ──");
    let cb = CheckBuilder::new("CRY-01", "crypto", Category::Crypto, Severity::Critical)
        .remediation("Regenerate cookie secret: deploy/rotate_cookie_secret.sh");
    let secret_path = cfg.cookie_secret.display().to_string();

    let (code, content) = sudo_cmd("root", &format!("cat {} 2>/dev/null", secret_path));
    if code != 0 || content.trim().is_empty() {
        results.push(cb.fail("Cookie secret file not readable or empty", &secret_path));
        return;
    }

    let secret = content.trim();
    let raw = hex_decode(secret);
    let byte_len = raw.as_ref().map_or(secret.len(), Vec::len);

    if byte_len < 32 {
        results.push(cb.fail(
            &format!("Cookie secret too short: {byte_len} bytes (need >= 32)"),
            &format!("length={byte_len}"),
        ));
        return;
    }

    let data = raw.as_deref().unwrap_or(secret.as_bytes());
    let ent = shannon_entropy(data);
    if ent < 4.0 {
        results.push(cb.fail(
            &format!("Cookie secret low entropy: {ent:.2} bits/byte (need >= 4.0)"),
            &format!("entropy={ent:.2}, length={byte_len}"),
        ));
    } else {
        results.push(cb.pass(
            &format!("Cookie secret entropy OK: {ent:.2} bits/byte, {byte_len} bytes"),
            &format!("entropy={ent:.2}, length={byte_len}"),
        ));
    }
}

pub(crate) fn cry_02_cookie_age(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-02: Cookie Secret Age ──");
    let cb = CheckBuilder::new("CRY-02", "crypto", Category::Crypto, Severity::Medium)
        .remediation("Rotate monthly: sudo bash deploy/rotate_cookie_secret.sh");
    let secret_path = cfg.cookie_secret.display().to_string();

    let (code, stat_out) = sudo_cmd("root", &format!("stat -c '%Y' {} 2>/dev/null", secret_path));
    if code != 0 {
        results.push(cb.fail("Cannot stat cookie secret file", &secret_path));
        return;
    }

    if let Ok(mtime) = stat_out.trim().parse::<u64>() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let age_days = (now.saturating_sub(mtime)) / 86400;
        if age_days > 90 {
            results.push(cb.dark(
                &format!("Cookie secret is {age_days} days old (> 90 days)"),
                &format!("age_days={age_days}"),
            ));
        } else {
            results.push(cb.pass(
                &format!("Cookie secret is {age_days} days old (within 90-day window)"),
                &format!("age_days={age_days}"),
            ));
        }
    } else {
        results.push(cb.fail("Cannot parse cookie secret mtime", stat_out.trim()));
    }
}

pub(crate) fn cry_03_cookie_permissions(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-03: Cookie Secret Permissions ──");
    let cb = CheckBuilder::new("CRY-03", "crypto", Category::Crypto, Severity::High)
        .remediation("chmod 600 and chown root the cookie secret file");
    let secret_path = cfg.cookie_secret.display().to_string();

    let (code, stat_out) = sudo_cmd("root", &format!("stat -c '%a %U' {} 2>/dev/null", secret_path));
    if code != 0 {
        results.push(cb.fail("Cannot stat cookie secret file", &secret_path));
        return;
    }

    let parts: Vec<&str> = stat_out.trim().split_whitespace().collect();
    let perms = parts.first().unwrap_or(&"???");
    let owner = parts.get(1).unwrap_or(&"???");

    let perms_ok = *perms == "600" || *perms == "400";
    let owner_ok = *owner == "root" || *owner == "irongate";

    if perms_ok && owner_ok {
        results.push(cb.pass(
            &format!("Cookie secret permissions OK: mode={perms}, owner={owner}"),
            &format!("{perms} {owner}"),
        ));
    } else {
        results.push(cb.fail(
            &format!("Cookie secret permissions unsafe: mode={perms}, owner={owner} (need 600, root)"),
            &format!("{perms} {owner}"),
        ));
    }
}

pub(crate) fn cry_13_sensitive_file_permissions(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-13: Sensitive File Permission Sweep ──");

    let cookie_path = cfg.cookie_secret.display().to_string();
    let sqlite_path = cfg.sqlite.display().to_string();
    let cf_config_path = format!("{}/config.yml", cfg.cloudflared_dir.display());
    let sensitive_files: Vec<(&str, &str)> = vec![
        (&cookie_path, "Cookie secret"),
        (&sqlite_path, "JupyterHub database"),
        (&cf_config_path, "Cloudflare tunnel config"),
    ];

    for (path, label) in &sensitive_files {
        let id = format!("CRY-13-{}", label.replace(' ', "_").to_lowercase());
        let cb = CheckBuilder::new(&id, "crypto", Category::Crypto, Severity::High)
            .remediation(&format!("chmod 600 {path}"));

        let (code, stat_out) = sudo_cmd("root", &format!("stat -c '%a' {} 2>/dev/null", path));
        if code != 0 {
            results.push(cb.pass(&format!("{label}: file not found (acceptable)"), path));
            continue;
        }

        let perms = stat_out.trim();
        let mode: u32 = perms.parse().unwrap_or(777);
        let world_readable = mode % 10 >= 4;
        let group_readable = (mode / 10) % 10 >= 4;

        if world_readable {
            results.push(cb.fail(&format!("{label}: world-readable ({perms})"), &format!("{path} mode={perms}")));
        } else if group_readable && mode != 640 {
            results.push(cb.dark(&format!("{label}: group-readable ({perms}) — review group membership"), &format!("{path} mode={perms}")));
        } else {
            results.push(cb.pass(&format!("{label}: permissions OK ({perms})"), &format!("{path} mode={perms}")));
        }
    }

    let (code, creds_out) = sudo_cmd("root", &format!("ls -la {}/*.json 2>/dev/null", cfg.cloudflared_dir.display()));
    if code == 0 && !creds_out.trim().is_empty() {
        let cb = CheckBuilder::new("CRY-13-tunnel_creds", "crypto", Category::Crypto, Severity::Critical)
            .remediation("chmod 600 ~/.cloudflared/*.json");
        let has_world = creds_out.lines().any(|l| {
            let chars: Vec<char> = l.chars().collect();
            chars.len() > 9 && (chars[7] == 'r' || chars[8] == 'w')
        });
        if has_world {
            results.push(cb.fail("Tunnel credential files world-readable", &creds_out[..120.min(creds_out.len())]));
        } else {
            results.push(cb.pass("Tunnel credential files properly restricted", "Not world-readable"));
        }
    }
}
