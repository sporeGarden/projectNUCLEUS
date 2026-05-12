use crate::check::*;
use crate::net::sudo_cmd;
use super::gate_home;

pub(crate) fn cry_04_api_token_entropy(results: &mut Vec<CheckResult>) {
    println!("── CRY-04: API Token Entropy ──");
    let cb = CheckBuilder::new("CRY-04", "crypto", Category::Crypto, Severity::High)
        .remediation("JupyterHub generates UUID4 tokens by default — ensure no custom overrides");

    let db = gate_home().join("jupyterhub/jupyterhub.sqlite");
    let (code, out) = sudo_cmd("root",
        &format!("sqlite3 {} \"SELECT prefix FROM api_tokens LIMIT 5\" 2>/dev/null", db.display()));
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

pub(crate) fn cry_05_shadow_hash_algorithm(results: &mut Vec<CheckResult>) {
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

pub(crate) fn cry_06_shadow_hash_strength(results: &mut Vec<CheckResult>) {
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

pub(crate) fn cry_11_master_key_present(results: &mut Vec<CheckResult>) {
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
