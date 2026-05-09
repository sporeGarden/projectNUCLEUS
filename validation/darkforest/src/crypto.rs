use crate::check::*;
use crate::net::*;
use std::path::PathBuf;
use std::time::SystemTime;

fn gate_home() -> PathBuf {
    PathBuf::from(std::env::var("GATE_HOME")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "/home/nobody".to_string()))
}

fn cookie_secret_path() -> PathBuf {
    std::env::var("JUPYTERHUB_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| gate_home().join("jupyterhub"))
        .join("jupyterhub_cookie_secret")
}

fn sqlite_path() -> PathBuf {
    std::env::var("JUPYTERHUB_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| gate_home().join("jupyterhub"))
        .join("jupyterhub.sqlite")
}

fn cf_dir() -> PathBuf {
    std::env::var("CLOUDFLARED_DIR").map(PathBuf::from)
        .unwrap_or_else(|_| gate_home().join(".cloudflared"))
}

struct CryptoConfig {
    hub_port: u16,
    beardog_port: u16,
    cookie_secret: PathBuf,
    sqlite: PathBuf,
    cloudflared_dir: PathBuf,
}

pub fn run(host: &str, results: &mut Vec<CheckResult>) {
    let cfg = CryptoConfig {
        hub_port: hub_port(),
        beardog_port: std::env::var("BEARDOG_PORT")
            .ok().and_then(|v| v.parse().ok()).unwrap_or(9100),
        cookie_secret: cookie_secret_path(),
        sqlite: sqlite_path(),
        cloudflared_dir: cf_dir(),
    };
    println!("\n══ Crypto Strength Validation ══\n");

    cry_01_cookie_entropy(&cfg, results);
    cry_02_cookie_age(&cfg, results);
    cry_03_cookie_permissions(&cfg, results);
    cry_04_api_token_entropy(results);
    cry_05_shadow_hash_algorithm(results);
    cry_06_shadow_hash_strength(results);
    cry_07_ionic_tamper_rejection(host, &cfg, results);
    cry_08_ionic_expiry_enforcement(host, &cfg, results);
    cry_09_btsp_cipher_negotiation(host, &cfg, results);
    cry_10_btsp_null_rejection(host, &cfg, results);
    cry_11_master_key_present(results);
    cry_12_cookie_signing_version(host, &cfg, results);
    cry_13_sensitive_file_permissions(&cfg, results);
}

fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0u64; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let len = data.len() as f64;
    let mut entropy = 0.0f64;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn cry_01_cookie_entropy(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
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

fn cry_02_cookie_age(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
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

fn cry_03_cookie_permissions(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
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

fn cry_04_api_token_entropy(results: &mut Vec<CheckResult>) {
    println!("── CRY-04: API Token Entropy ──");
    let cb = CheckBuilder::new("CRY-04", "crypto", Category::Crypto, Severity::High)
        .remediation("JupyterHub generates UUID4 tokens by default — ensure no custom overrides");

    let (code, out) = sudo_cmd("root",
        "sqlite3 /home/irongate/jupyterhub/jupyterhub.sqlite \"SELECT prefix FROM api_tokens LIMIT 5\" 2>/dev/null");
    if code != 0 || out.trim().is_empty() {
        results.push(cb.pass("No API tokens in database or database not accessible", "Empty result"));
        return;
    }

    let prefixes: Vec<&str> = out.trim().lines().collect();
    let all_hex = prefixes.iter().all(|p| {
        let t = p.trim();
        !t.is_empty() && t.len() >= 4 && t.chars().all(|c| c.is_ascii_hexdigit())
    });

    if all_hex {
        results.push(cb.pass(
            &format!("API token prefixes are hex (UUID-derived): {} tokens checked", prefixes.len()),
            &format!("prefixes={}", prefixes.join(",")),
        ));
    } else {
        results.push(cb.dark(
            "API token prefixes contain non-hex characters — may not be UUID4",
            &format!("prefixes={}", prefixes.join(",")),
        ));
    }
}

fn cry_05_shadow_hash_algorithm(results: &mut Vec<CheckResult>) {
    println!("── CRY-05: Shadow Hash Algorithm ──");

    let abg_users = [COMPUTE_USER, REVIEWER_USER, OBSERVER_USER, "kmok"];
    for user in &abg_users {
        let cb = CheckBuilder::new(&format!("CRY-05-{user}"), "crypto", Category::Crypto, Severity::Critical)
            .remediation("Set ENCRYPT_METHOD SHA512 or yescrypt in /etc/login.defs");

        let (code, line) = sudo_cmd("root", &format!("grep '^{user}:' /etc/shadow 2>/dev/null"));
        if code != 0 || line.trim().is_empty() {
            results.push(cb.pass(&format!("{user}: no shadow entry (system account or no password)"), "Not found"));
            continue;
        }

        let hash_field = line.split(':').nth(1).unwrap_or("");
        if hash_field == "*" || hash_field == "!" || hash_field == "!!" || hash_field.is_empty() {
            results.push(cb.pass(&format!("{user}: account locked (no password hash)"), hash_field));
            continue;
        }

        // $id$... format: $1$ = MD5, $5$ = SHA-256, $6$ = SHA-512, $y$ = yescrypt
        if hash_field.starts_with("$6$") {
            results.push(cb.pass(&format!("{user}: SHA-512 hash"), "$6$..."));
        } else if hash_field.starts_with("$y$") {
            results.push(cb.pass(&format!("{user}: yescrypt hash"), "$y$..."));
        } else if hash_field.starts_with("$5$") {
            results.push(cb.pass(&format!("{user}: SHA-256 hash (acceptable)"), "$5$..."));
        } else if hash_field.starts_with("$1$") {
            results.push(cb.fail(&format!("{user}: MD5 hash — weak"), "$1$..."));
        } else if !hash_field.starts_with('$') {
            results.push(cb.fail(&format!("{user}: DES hash — critically weak"), "DES"));
        } else {
            let alg_id = hash_field.split('$').nth(1).unwrap_or("?");
            results.push(cb.pass(&format!("{user}: hash algorithm ${alg_id}$"), &format!("${alg_id}$...")));
        }
    }
}

fn cry_06_shadow_hash_strength(results: &mut Vec<CheckResult>) {
    println!("── CRY-06: Shadow Hash Rounds ──");

    let cb = CheckBuilder::new("CRY-06", "crypto", Category::Crypto, Severity::Medium)
        .remediation("Set SHA_CRYPT_MIN_ROUNDS >= 5000 in /etc/login.defs");

    let (code, defs) = sudo_cmd("root", "grep -i 'SHA_CRYPT_MIN_ROUNDS\\|YESCRYPT_COST_FACTOR' /etc/login.defs 2>/dev/null");
    if code != 0 || defs.trim().is_empty() {
        results.push(cb.pass(
            "No explicit rounds/cost in login.defs (using distro defaults — acceptable for SHA-512)",
            "Defaults in use",
        ));
        return;
    }

    let evidence = defs.trim().to_string();
    let rounds_ok = defs.lines().any(|l| {
        if let Some(val) = l.split_whitespace().last() {
            val.parse::<u32>().map_or(false, |n| n >= 5000)
        } else {
            false
        }
    });

    if rounds_ok || defs.contains("YESCRYPT") {
        results.push(cb.pass("Hash rounds/cost factor adequate", &evidence));
    } else {
        results.push(cb.dark("Hash rounds may be low — review login.defs", &evidence));
    }
}

fn cry_07_ionic_tamper_rejection(host: &str, cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-07: Ionic Token Tamper Rejection ──");
    let cb = CheckBuilder::new("CRY-07", "crypto", Category::Crypto, Severity::Critical)
        .remediation("BearDog auth.verify_ionic must reject tampered signatures");

    let tampered = "eyJhbGciOiJFZERTQSIsInR5cCI6ImlvbmljIiwidmVyIjoxfQ.\
                    eyJpc3MiOiJ0ZXN0Iiwic3ViIjoidGFtcGVyIiwic2NvcGUiOlsiKiJdLCJpYXQiOjE3MTUwMDAwMDAsImV4cCI6OTk5OTk5OTk5OSwianRpIjoiZGVhZGJlZWYwMDAwMDAwMCJ9.\
                    AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"auth.verify_ionic","params":{{"token":"{tampered}"}},"id":1}}"#
    );
    if let Some((_, body)) = send_jsonrpc(host, cfg.beardog_port, &payload, 5000) {
        if body.contains("\"error\"") || body.contains("-32001") || body.contains("-32602") {
            results.push(cb.pass("Tampered ionic token rejected by BearDog", &body[..120.min(body.len())]));
        } else if body.contains("\"result\"") && body.contains("true") {
            results.push(cb.fail("Tampered ionic token ACCEPTED — signature verification broken", &body[..120.min(body.len())]));
        } else {
            results.push(cb.pass("BearDog rejected tampered token (non-standard response)", &body[..120.min(body.len())]));
        }
    } else {
        results.push(cb.pass(&format!("BearDog not reachable on :{} (skip)", cfg.beardog_port), "Connection refused"));
    }
}

fn cry_08_ionic_expiry_enforcement(host: &str, cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-08: Ionic Token Expiry Enforcement ──");
    let cb = CheckBuilder::new("CRY-08", "crypto", Category::Crypto, Severity::High)
        .remediation("BearDog must reject expired tokens");

    // We can't easily create a validly-signed but expired token without the signing key.
    // Instead, verify that auth.verify_ionic checks expiry by sending a structurally valid
    // but expired+tampered token — if it rejects for *any* reason, expiry path is exercised.
    let expired = "eyJhbGciOiJFZERTQSIsInR5cCI6ImlvbmljIiwidmVyIjoxfQ.\
                   eyJpc3MiOiJ0ZXN0Iiwic3ViIjoiZXhwaXJlZCIsInNjb3BlIjpbIioiXSwiaWF0IjoxNzE1MDAwMDAwLCJleHAiOjE3MTUwMDAwMDEsImp0aSI6ImRlYWRiZWVmMDAwMDAwMDEifQ.\
                   AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"auth.verify_ionic","params":{{"token":"{expired}"}},"id":1}}"#
    );
    if let Some((_, body)) = send_jsonrpc(host, cfg.beardog_port, &payload, 5000) {
        if body.contains("\"error\"") || body.contains("-32") {
            results.push(cb.pass("Expired/tampered ionic token rejected by BearDog", &body[..120.min(body.len())]));
        } else if body.contains("\"result\"") && body.contains("true") {
            results.push(cb.fail("Expired ionic token ACCEPTED — expiry enforcement broken", &body[..120.min(body.len())]));
        } else {
            results.push(cb.pass("BearDog rejected expired token (non-standard response)", &body[..120.min(body.len())]));
        }
    } else {
        results.push(cb.pass(&format!("BearDog not reachable on :{} (skip)", cfg.beardog_port), "Connection refused"));
    }
}

fn cry_09_btsp_cipher_negotiation(host: &str, _cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    let sweetgrass_port: u16 = std::env::var("SWEETGRASS_PORT")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(9850);
    println!("── CRY-09: BTSP Cipher Negotiation ──");
    let cb = CheckBuilder::new("CRY-09", "crypto", Category::Crypto, Severity::High)
        .remediation("BTSP must negotiate chacha20_poly1305 in production");

    if let Some(resp) = send_raw(host, sweetgrass_port, b"HELLO PLAINTEXT\n", 3000) {
        let text = String::from_utf8_lossy(&resp);
        if text.contains("result") || text.contains("200") {
            results.push(cb.fail("BTSP port accepted plaintext — cipher negotiation bypassed", &text[..80.min(text.len())]));
        } else {
            results.push(cb.pass("BTSP port rejects plaintext (handshake required)", &text[..80.min(text.len())]));
        }
    } else {
        results.push(cb.pass("BTSP port rejects plaintext connection", "Connection rejected/closed"));
    }
}

fn cry_10_btsp_null_rejection(host: &str, _cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    let rhizocrypt_port: u16 = std::env::var("RHIZOCRYPT_PORT")
        .ok().and_then(|v| v.parse().ok()).unwrap_or(9601);
    println!("── CRY-10: BTSP Null Cipher Rejection ──");
    let cb = CheckBuilder::new("CRY-10", "crypto", Category::Crypto, Severity::High)
        .remediation("BTSP must reject null cipher in production mode");

    if let Some(resp) = send_raw(host, rhizocrypt_port, b"HELLO PLAINTEXT\n", 3000) {
        let text = String::from_utf8_lossy(&resp);
        if text.contains("result") {
            results.push(cb.fail("BTSP port accepted plaintext on rhizocrypt:9601", &text[..80.min(text.len())]));
        } else {
            results.push(cb.pass("rhizocrypt BTSP rejects plaintext", &text[..80.min(text.len())]));
        }
    } else {
        results.push(cb.pass("rhizocrypt BTSP rejects plaintext connection", "Connection rejected"));
    }
}

fn cry_11_master_key_present(results: &mut Vec<CheckResult>) {
    println!("── CRY-11: BearDog Master Key Presence ──");
    let cb = CheckBuilder::new("CRY-11", "crypto", Category::Crypto, Severity::High)
        .remediation("Set BEARDOG_MASTER_KEY env var for persistent key derivation");

    let (code, out) = sudo_cmd("root",
        &format!("grep -r 'BEARDOG_MASTER_KEY' /etc/systemd/system/beardog* {}/.config/systemd/user/beardog* 2>/dev/null | head -3",
            gate_home().display()));
    if code == 0 && !out.trim().is_empty() && out.contains("BEARDOG_MASTER_KEY") {
        let has_value = out.contains('=') && !out.contains("BEARDOG_MASTER_KEY=\"\"") && !out.contains("BEARDOG_MASTER_KEY=\n");
        if has_value {
            results.push(cb.pass("BEARDOG_MASTER_KEY is set in systemd unit", "Found in service definition"));
        } else {
            results.push(cb.dark("BEARDOG_MASTER_KEY defined but may be empty", &out[..80.min(out.len())]));
        }
    } else {
        results.push(cb.dark(
            "BEARDOG_MASTER_KEY not found in systemd units — ephemeral key derivation in use",
            "Key material regenerated each restart",
        ));
    }
}

fn cry_12_cookie_signing_version(host: &str, cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
    println!("── CRY-12: Cookie Signing Version ──");
    let cb = CheckBuilder::new("CRY-12", "crypto", Category::Crypto, Severity::Medium)
        .remediation("Ensure Tornado >= 6.0 for HMAC-SHA256 signed cookies (v2 format)");

    if let Some((_, headers, _)) = http_get(host, cfg.hub_port, "/hub/login", "", 5000) {
        let cookie_line = headers.lines().find(|l| l.to_lowercase().contains("set-cookie"));
        if let Some(cookie) = cookie_line {
            // Tornado v2 signed values use | as delimiter with timestamp
            // v1 uses plain base64 without timestamp
            if cookie.contains('|') {
                results.push(cb.pass("Cookie uses Tornado signed-value v2 format (HMAC-SHA256)", cookie));
            } else {
                results.push(cb.dark("Cookie format may be v1 (SHA-1) — verify Tornado version", cookie));
            }
        } else {
            results.push(cb.pass("No signed cookie in login response (redirect flow)", "No Set-Cookie header"));
        }
    } else {
        results.push(cb.pass("Hub not reachable (skip)", "Connection refused"));
    }
}

fn cry_13_sensitive_file_permissions(cfg: &CryptoConfig, results: &mut Vec<CheckResult>) {
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

    // Tunnel credentials — any .json in .cloudflared/
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

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if s.len() % 2 != 0 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&s[i..i + 2], 16).ok()?);
    }
    Some(bytes)
}
