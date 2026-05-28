use super::{CryptoConfig, hex_decode, shannon_entropy};
use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net::sudo_cmd;
use std::time::SystemTime;

fn cookie_permissions_acceptable(perms: &str, owner: &str) -> bool {
    let perms_ok = perms == "600" || perms == "400";
    let owner_ok = owner == "root" || owner == "irongate";
    perms_ok && owner_ok
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SensitiveModeVerdict {
    Ok,
    WorldReadable,
    GroupReadable,
}

fn classify_sensitive_file_mode(perms: &str) -> SensitiveModeVerdict {
    let mode: u32 = perms.parse().unwrap_or(777);
    let world_readable = mode % 10 >= 4;
    let group_readable = (mode / 10) % 10 >= 4;

    if world_readable {
        SensitiveModeVerdict::WorldReadable
    } else if group_readable && mode != 640 {
        SensitiveModeVerdict::GroupReadable
    } else {
        SensitiveModeVerdict::Ok
    }
}

fn ls_line_world_accessible(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    chars.len() > 9 && (chars[7] == 'r' || chars[8] == 'w')
}

fn evaluate_cookie_secret_content(content: &str) -> Result<(f64, usize), &'static str> {
    let secret = content.trim();
    if secret.is_empty() {
        return Err("empty");
    }

    let raw = hex_decode(secret);
    let byte_len = raw.as_ref().map_or(secret.len(), Vec::len);

    if byte_len < 32 {
        return Err("short");
    }

    let data = raw.as_deref().unwrap_or(secret.as_bytes());
    let ent = shannon_entropy(data);
    if ent < 4.0 {
        return Err("low_entropy");
    }

    Ok((ent, byte_len))
}

pub fn cry_01_cookie_entropy(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-01: Cookie Secret Entropy ──");
    let cb = CheckBuilder::new("CRY-01", "crypto", Category::Crypto, Severity::Critical)
        .remediation("Regenerate cookie secret: deploy/rotate_cookie_secret.sh");
    let secret_path = cfg.cookie_secret.display().to_string();

    let (code, content) = sudo_cmd("root", &format!("cat {secret_path} 2>/dev/null"));
    if code != 0 || content.trim().is_empty() {
        results.push(cb.fail("Cookie secret file not readable or empty", &secret_path));
        return;
    }

    match evaluate_cookie_secret_content(&content) {
        Err("empty") => {
            results.push(cb.fail("Cookie secret file not readable or empty", &secret_path));
        }
        Err("short") => {
            let secret = content.trim();
            let byte_len = hex_decode(secret).map_or(secret.len(), |v| v.len());
            results.push(cb.fail(
                &format!("Cookie secret too short: {byte_len} bytes (need >= 32)"),
                &format!("length={byte_len}"),
            ));
        }
        Err("low_entropy") => {
            let secret = content.trim();
            let raw = hex_decode(secret);
            let byte_len = raw.as_ref().map_or(secret.len(), Vec::len);
            let data = raw.as_deref().unwrap_or(secret.as_bytes());
            let ent = shannon_entropy(data);
            results.push(cb.fail(
                &format!("Cookie secret low entropy: {ent:.2} bits/byte (need >= 4.0)"),
                &format!("entropy={ent:.2}, length={byte_len}"),
            ));
        }
        Ok((ent, byte_len)) => {
            results.push(cb.pass(
                &format!("Cookie secret entropy OK: {ent:.2} bits/byte, {byte_len} bytes"),
                &format!("entropy={ent:.2}, length={byte_len}"),
            ));
        }
        Err(_) => unreachable!(),
    }
}

pub fn cry_02_cookie_age(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-02: Cookie Secret Age ──");
    let cb = CheckBuilder::new("CRY-02", "crypto", Category::Crypto, Severity::Medium)
        .remediation("Rotate monthly: sudo bash deploy/rotate_cookie_secret.sh");
    let secret_path = cfg.cookie_secret.display().to_string();

    let (code, stat_out) = sudo_cmd("root", &format!("stat -c '%Y' {secret_path} 2>/dev/null"));
    if code != 0 {
        results.push(cb.fail("Cannot stat cookie secret file", &secret_path));
        return;
    }

    if let Ok(mtime) = stat_out.trim().parse::<u64>() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
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

pub fn cry_03_cookie_permissions(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-03: Cookie Secret Permissions ──");
    let cb = CheckBuilder::new("CRY-03", "crypto", Category::Crypto, Severity::High)
        .remediation("chmod 600 and chown root the cookie secret file");
    let secret_path = cfg.cookie_secret.display().to_string();

    let (code, stat_out) = sudo_cmd(
        "root",
        &format!("stat -c '%a %U' {secret_path} 2>/dev/null"),
    );
    if code != 0 {
        results.push(cb.fail("Cannot stat cookie secret file", &secret_path));
        return;
    }

    let parts: Vec<&str> = stat_out.split_whitespace().collect();
    let perms = parts.first().unwrap_or(&"???");
    let owner = parts.get(1).unwrap_or(&"???");

    if cookie_permissions_acceptable(perms, owner) {
        results.push(cb.pass(
            &format!("Cookie secret permissions OK: mode={perms}, owner={owner}"),
            &format!("{perms} {owner}"),
        ));
    } else {
        results.push(cb.fail(
            &format!(
                "Cookie secret permissions unsafe: mode={perms}, owner={owner} (need 600, root)"
            ),
            &format!("{perms} {owner}"),
        ));
    }
}

pub fn cry_13_sensitive_file_permissions(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
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

        let (code, stat_out) = sudo_cmd("root", &format!("stat -c '%a' {path} 2>/dev/null"));
        if code != 0 {
            results.push(cb.pass(&format!("{label}: file not found (acceptable)"), path));
            continue;
        }

        let perms = stat_out.trim();
        match classify_sensitive_file_mode(perms) {
            SensitiveModeVerdict::WorldReadable => {
                results.push(cb.fail(
                    &format!("{label}: world-readable ({perms})"),
                    &format!("{path} mode={perms}"),
                ));
            }
            SensitiveModeVerdict::GroupReadable => {
                results.push(cb.dark(
                    &format!("{label}: group-readable ({perms}) — review group membership"),
                    &format!("{path} mode={perms}"),
                ));
            }
            SensitiveModeVerdict::Ok => {
                results.push(cb.pass(
                    &format!("{label}: permissions OK ({perms})"),
                    &format!("{path} mode={perms}"),
                ));
            }
        }
    }

    let (code, creds_out) = sudo_cmd(
        "root",
        &format!(
            "ls -la {}/*.json 2>/dev/null",
            cfg.cloudflared_dir.display()
        ),
    );
    if code == 0 && !creds_out.trim().is_empty() {
        let cb = CheckBuilder::new(
            "CRY-13-tunnel_creds",
            "crypto",
            Category::Crypto,
            Severity::Critical,
        )
        .remediation("chmod 600 ~/.cloudflared/*.json");
        let has_world = creds_out.lines().any(ls_line_world_accessible);
        if has_world {
            results.push(cb.fail(
                "Tunnel credential files world-readable",
                &creds_out[..120.min(creds_out.len())],
            ));
        } else {
            results.push(cb.pass(
                "Tunnel credential files properly restricted",
                "Not world-readable",
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn cookie_permissions_acceptable_for_root_600() {
        assert!(cookie_permissions_acceptable("600", "root"));
    }

    #[test]
    fn cookie_permissions_acceptable_for_irongate_400() {
        assert!(cookie_permissions_acceptable("400", "irongate"));
    }

    #[test]
    fn cookie_permissions_rejects_world_readable() {
        assert!(!cookie_permissions_acceptable("644", "root"));
    }

    #[test]
    fn cookie_permissions_rejects_wrong_owner() {
        assert!(!cookie_permissions_acceptable("600", "nobody"));
    }

    #[test]
    fn classify_sensitive_file_mode_secure() {
        assert_eq!(
            classify_sensitive_file_mode("600"),
            SensitiveModeVerdict::Ok
        );
    }

    #[test]
    fn classify_sensitive_file_mode_world_readable() {
        assert_eq!(
            classify_sensitive_file_mode("644"),
            SensitiveModeVerdict::WorldReadable
        );
    }

    #[test]
    fn classify_sensitive_file_mode_group_readable_not_640() {
        assert_eq!(
            classify_sensitive_file_mode("660"),
            SensitiveModeVerdict::GroupReadable
        );
    }

    #[test]
    fn classify_sensitive_file_mode_640_is_ok() {
        assert_eq!(
            classify_sensitive_file_mode("640"),
            SensitiveModeVerdict::Ok
        );
    }

    #[test]
    fn ls_line_world_accessible_detects_world_read() {
        let line = "-rw-r--r-- 1 root root 123 Jan 1 00:00 cred.json";
        assert!(ls_line_world_accessible(line));
    }

    #[test]
    fn ls_line_world_accessible_secure_line() {
        let line = "-rw------- 1 root root 123 Jan 1 00:00 cred.json";
        assert!(!ls_line_world_accessible(line));
    }

    #[test]
    fn evaluate_cookie_secret_content_rejects_empty() {
        assert_eq!(evaluate_cookie_secret_content("   \n"), Err("empty"));
    }

    #[test]
    fn evaluate_cookie_secret_content_rejects_short_secret() {
        assert_eq!(evaluate_cookie_secret_content("abcd"), Err("short"));
    }

    #[test]
    fn evaluate_cookie_secret_content_rejects_low_entropy() {
        let secret = "aa".repeat(32);
        assert_eq!(evaluate_cookie_secret_content(&secret), Err("low_entropy"));
    }

    #[test]
    fn evaluate_cookie_secret_content_accepts_high_entropy_hex() {
        let secret = (0..32u32)
            .map(|i| format!("{:02x}", (i.wrapping_mul(37).wrapping_add(13)) & 0xFF))
            .collect::<String>();
        let result = evaluate_cookie_secret_content(&secret);
        assert!(result.is_ok());
        let (ent, byte_len) = result.unwrap();
        assert_eq!(byte_len, 32);
        assert!(ent >= 4.0);
    }

    #[test]
    fn evaluate_cookie_secret_from_temp_file() {
        let dir = std::env::temp_dir().join("darkforest_test_cookie_secret");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("jupyterhub_cookie_secret");
        let secret = (0..32u32)
            .map(|i| format!("{:02x}", (i.wrapping_mul(53).wrapping_add(7)) & 0xFF))
            .collect::<String>();
        fs::write(&path, &secret).expect("write secret");

        let content = fs::read_to_string(&path).expect("read secret");
        let result = evaluate_cookie_secret_content(&content);
        assert!(result.is_ok());

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&dir);
    }
}
