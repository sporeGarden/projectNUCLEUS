use super::gate_home;
use crate::check::{
    Category, CheckBuilder, CheckResult, Severity, compute_user, observer_user, reviewer_user,
};
use crate::net::sudo_cmd;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ShadowHashVerdict {
    Locked,
    Sha512,
    Yescrypt,
    Sha256,
    Md5Weak,
    DesWeak,
    Other(String),
}

fn classify_shadow_hash(hash_field: &str) -> ShadowHashVerdict {
    if hash_field == "*" || hash_field == "!" || hash_field == "!!" || hash_field.is_empty() {
        ShadowHashVerdict::Locked
    } else if hash_field.starts_with("$6$") {
        ShadowHashVerdict::Sha512
    } else if hash_field.starts_with("$y$") {
        ShadowHashVerdict::Yescrypt
    } else if hash_field.starts_with("$5$") {
        ShadowHashVerdict::Sha256
    } else if hash_field.starts_with("$1$") {
        ShadowHashVerdict::Md5Weak
    } else if !hash_field.starts_with('$') {
        ShadowHashVerdict::DesWeak
    } else {
        let alg_id = hash_field.split('$').nth(1).unwrap_or("?").to_string();
        ShadowHashVerdict::Other(alg_id)
    }
}

fn hash_rounds_adequate(defs: &str) -> bool {
    defs.lines().any(|l| {
        l.split_whitespace()
            .last()
            .is_some_and(|val| val.parse::<u32>().is_ok_and(|n| n >= 5000))
    }) || defs.contains("YESCRYPT")
}

fn api_token_prefixes_all_hex(prefixes: &[&str]) -> bool {
    prefixes.iter().all(|p| {
        let t = p.trim();
        !t.is_empty() && t.len() >= 4 && t.chars().all(|c| c.is_ascii_hexdigit())
    })
}

fn beardog_master_key_has_value(out: &str) -> bool {
    out.contains('=')
        && !out.contains("BEARDOG_MASTER_KEY=\"\"")
        && !out.contains("BEARDOG_MASTER_KEY=\n")
}

pub fn cry_04_api_token_entropy(results: &mut Vec<CheckResult>) {
    println!("── CRY-04: API Token Entropy ──");
    let cb = CheckBuilder::new("CRY-04", "crypto", Category::Crypto, Severity::High)
        .remediation("JupyterHub generates UUID4 tokens by default — ensure no custom overrides");

    let db = gate_home().join("jupyterhub/jupyterhub.sqlite");
    let (code, out) = sudo_cmd(
        "root",
        &format!(
            "sqlite3 {} \"SELECT prefix FROM api_tokens LIMIT 5\" 2>/dev/null",
            db.display()
        ),
    );
    if code != 0 || out.trim().is_empty() {
        results.push(cb.pass(
            "No API tokens in database or database not accessible",
            "Empty result",
        ));
        return;
    }

    let prefixes: Vec<&str> = out.trim().lines().collect();
    let all_hex = api_token_prefixes_all_hex(&prefixes);

    if all_hex {
        results.push(cb.pass(
            &format!(
                "API token prefixes are hex (UUID-derived): {} tokens checked",
                prefixes.len()
            ),
            &format!("prefixes={}", prefixes.join(",")),
        ));
    } else {
        results.push(cb.dark(
            "API token prefixes contain non-hex characters — may not be UUID4",
            &format!("prefixes={}", prefixes.join(",")),
        ));
    }
}

pub fn cry_05_shadow_hash_algorithm(results: &mut Vec<CheckResult>) {
    println!("── CRY-05: Shadow Hash Algorithm ──");

    let compute = compute_user();
    let reviewer = reviewer_user();
    let observer = observer_user();
    let abg_users = [&compute, &reviewer, &observer, "kmok"];
    for user in &abg_users {
        let cb = CheckBuilder::new(
            &format!("CRY-05-{user}"),
            "crypto",
            Category::Crypto,
            Severity::Critical,
        )
        .remediation("Set ENCRYPT_METHOD SHA512 or yescrypt in /etc/login.defs");

        let (code, line) = sudo_cmd("root", &format!("grep '^{user}:' /etc/shadow 2>/dev/null"));
        if code != 0 || line.trim().is_empty() {
            results.push(cb.pass(
                &format!("{user}: no shadow entry (system account or no password)"),
                "Not found",
            ));
            continue;
        }

        let hash_field = line.split(':').nth(1).unwrap_or("");
        match classify_shadow_hash(hash_field) {
            ShadowHashVerdict::Locked => {
                results.push(cb.pass(
                    &format!("{user}: account locked (no password hash)"),
                    hash_field,
                ));
            }
            ShadowHashVerdict::Sha512 => {
                results.push(cb.pass(&format!("{user}: SHA-512 hash"), "$6$..."));
            }
            ShadowHashVerdict::Yescrypt => {
                results.push(cb.pass(&format!("{user}: yescrypt hash"), "$y$..."));
            }
            ShadowHashVerdict::Sha256 => {
                results.push(cb.pass(&format!("{user}: SHA-256 hash (acceptable)"), "$5$..."));
            }
            ShadowHashVerdict::Md5Weak => {
                results.push(cb.fail(&format!("{user}: MD5 hash — weak"), "$1$..."));
            }
            ShadowHashVerdict::DesWeak => {
                results.push(cb.fail(&format!("{user}: DES hash — critically weak"), "DES"));
            }
            ShadowHashVerdict::Other(alg_id) => {
                results.push(cb.pass(
                    &format!("{user}: hash algorithm ${alg_id}$"),
                    &format!("${alg_id}$..."),
                ));
            }
        }
    }
}

pub fn cry_06_shadow_hash_strength(results: &mut Vec<CheckResult>) {
    println!("── CRY-06: Shadow Hash Rounds ──");

    let cb = CheckBuilder::new("CRY-06", "crypto", Category::Crypto, Severity::Medium)
        .remediation("Set SHA_CRYPT_MIN_ROUNDS >= 5000 in /etc/login.defs");

    let (code, defs) = sudo_cmd(
        "root",
        "grep -i 'SHA_CRYPT_MIN_ROUNDS\\|YESCRYPT_COST_FACTOR' /etc/login.defs 2>/dev/null",
    );
    if code != 0 || defs.trim().is_empty() {
        results.push(cb.pass(
            "No explicit rounds/cost in login.defs (using distro defaults — acceptable for SHA-512)",
            "Defaults in use",
        ));
        return;
    }

    let evidence = defs.trim().to_string();
    let rounds_ok = hash_rounds_adequate(&defs);

    if rounds_ok {
        results.push(cb.pass("Hash rounds/cost factor adequate", &evidence));
    } else {
        results.push(cb.dark("Hash rounds may be low — review login.defs", &evidence));
    }
}

pub fn cry_11_master_key_present(results: &mut Vec<CheckResult>) {
    println!("── CRY-11: BearDog Master Key Presence ──");
    let cb = CheckBuilder::new("CRY-11", "crypto", Category::Crypto, Severity::High)
        .remediation("Set BEARDOG_MASTER_KEY env var for persistent key derivation");

    let (code, out) = sudo_cmd(
        "root",
        &format!(
            "grep -r 'BEARDOG_MASTER_KEY' /etc/systemd/system/beardog* {}/.config/systemd/user/beardog* 2>/dev/null | head -3",
            gate_home().display()
        ),
    );
    if code == 0 && !out.trim().is_empty() && out.contains("BEARDOG_MASTER_KEY") {
        let has_value = beardog_master_key_has_value(&out);
        if has_value {
            results.push(cb.pass(
                "BEARDOG_MASTER_KEY is set in systemd unit",
                "Found in service definition",
            ));
        } else {
            results.push(cb.dark(
                "BEARDOG_MASTER_KEY defined but may be empty",
                &out[..80.min(out.len())],
            ));
        }
    } else {
        results.push(cb.dark(
            "BEARDOG_MASTER_KEY not found in systemd units — ephemeral key derivation in use",
            "Key material regenerated each restart",
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_shadow_hash_sha512() {
        assert_eq!(
            classify_shadow_hash("$6$rounds=5000$salt$hash"),
            ShadowHashVerdict::Sha512
        );
    }

    #[test]
    fn classify_shadow_hash_yescrypt() {
        assert_eq!(
            classify_shadow_hash("$y$j9T$rounds=5$salt$hash"),
            ShadowHashVerdict::Yescrypt
        );
    }

    #[test]
    fn classify_shadow_hash_sha256() {
        assert_eq!(
            classify_shadow_hash("$5$rounds=5000$salt$hash"),
            ShadowHashVerdict::Sha256
        );
    }

    #[test]
    fn classify_shadow_hash_md5_weak() {
        assert_eq!(
            classify_shadow_hash("$1$salt$hash"),
            ShadowHashVerdict::Md5Weak
        );
    }

    #[test]
    fn classify_shadow_hash_des_weak() {
        assert_eq!(classify_shadow_hash("abc123"), ShadowHashVerdict::DesWeak);
    }

    #[test]
    fn classify_shadow_hash_locked() {
        assert_eq!(classify_shadow_hash("*"), ShadowHashVerdict::Locked);
        assert_eq!(classify_shadow_hash("!"), ShadowHashVerdict::Locked);
        assert_eq!(classify_shadow_hash("!!"), ShadowHashVerdict::Locked);
    }

    #[test]
    fn classify_shadow_hash_other_algorithm() {
        assert_eq!(
            classify_shadow_hash("$2b$10$salt$hash"),
            ShadowHashVerdict::Other("2b".to_string())
        );
    }

    #[test]
    fn hash_rounds_adequate_with_high_rounds() {
        assert!(hash_rounds_adequate("SHA_CRYPT_MIN_ROUNDS 5000\n"));
    }

    #[test]
    fn hash_rounds_adequate_with_yescrypt() {
        assert!(hash_rounds_adequate("YESCRYPT_COST_FACTOR 5\n"));
    }

    #[test]
    fn hash_rounds_inadequate_with_low_rounds() {
        assert!(!hash_rounds_adequate("SHA_CRYPT_MIN_ROUNDS 1000\n"));
    }

    #[test]
    fn api_token_prefixes_all_hex_valid() {
        assert!(api_token_prefixes_all_hex(&["abcd", "ef012345"]));
    }

    #[test]
    fn api_token_prefixes_all_hex_rejects_short() {
        assert!(!api_token_prefixes_all_hex(&["abc"]));
    }

    #[test]
    fn api_token_prefixes_all_hex_rejects_non_hex() {
        assert!(!api_token_prefixes_all_hex(&["ghij"]));
    }

    #[test]
    fn beardog_master_key_has_value_when_set() {
        assert!(beardog_master_key_has_value(
            "Environment=BEARDOG_MASTER_KEY=secretvalue"
        ));
    }

    #[test]
    fn beardog_master_key_has_value_rejects_empty() {
        assert!(!beardog_master_key_has_value("BEARDOG_MASTER_KEY=\"\""));
    }
}
