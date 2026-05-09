use chacha20poly1305::aead::{Aead, KeyInit, OsRng};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use ed25519_dalek::SigningKey;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("encryption failed: {0}")]
    Encrypt(String),
    #[error("decryption failed: {0}")]
    Decrypt(String),
    #[error("key derivation failed: {0}")]
    KeyDerivation(String),
    #[error("{0}")]
    Other(String),
}

const KEY_FILE: &str = ".tunnelkeeper.key";

fn cloudflared_dir() -> PathBuf {
    if let Ok(d) = std::env::var("CLOUDFLARED_DIR") {
        PathBuf::from(d)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cloudflared")
    } else {
        PathBuf::from("/home").join(whoami()).join(".cloudflared")
    }
}

fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "nobody".to_string())
}

#[derive(Serialize, Deserialize)]
struct EncryptedBlob {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
    original_name: String,
}

/// BearDog-pattern credential encryption: Ed25519 key → derived ChaCha20-Poly1305 key
fn get_or_create_key() -> Result<[u8; 32], CryptoError> {
    let cf_dir = cloudflared_dir();
    let key_path = cf_dir.join(KEY_FILE);

    if key_path.exists() {
        let raw = fs::read(&key_path)?;
        if raw.len() < 32 {
            return Err(CryptoError::KeyDerivation("key file too short".into()));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&raw[..32]);
        Ok(key)
    } else {
        // Generate an Ed25519 signing key. The first 32 bytes of the secret
        // scalar serve as a symmetric key for ChaCha20-Poly1305 — same pattern
        // used by BearDog for at-rest credential wrapping.
        let signing = SigningKey::generate(&mut OsRng);
        let key_bytes = signing.to_bytes();
        fs::write(&key_path, key_bytes)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(key_bytes)
    }
}

fn find_creds_file(explicit: Option<&Path>) -> Result<PathBuf, CryptoError> {
    if let Some(p) = explicit {
        return Ok(p.to_path_buf());
    }
    let dir = cloudflared_dir();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".json") && !name_str.ends_with(".enc") && name_str != KEY_FILE {
            return Ok(entry.path());
        }
    }
    Err(CryptoError::Other(format!(
        "no credentials JSON found in {}",
        dir.display()
    )))
}

pub fn encrypt_creds(creds_path: Option<&Path>, json: bool) -> Result<(), CryptoError> {
    let path = find_creds_file(creds_path)?;
    let plaintext = fs::read(&path)?;
    let key = get_or_create_key()?;

    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;

    let blob = EncryptedBlob {
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        original_name: path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    };

    let enc_path = path.with_extension("json.enc");
    let serialized = serde_json::to_vec(&blob)
        .map_err(|e| CryptoError::Encrypt(e.to_string()))?;
    fs::write(&enc_path, serialized)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&enc_path, fs::Permissions::from_mode(0o600))?;
    }

    if json {
        println!(
            r#"{{"status":"encrypted","source":"{}","dest":"{}"}}"#,
            path.display(),
            enc_path.display()
        );
    } else {
        println!("Encrypted: {} → {}", path.display(), enc_path.display());
        println!("Key stored at: {}/{KEY_FILE}", cloudflared_dir().display());
        println!(
            "Original credentials can be removed once encryption is verified."
        );
    }
    Ok(())
}

pub fn decrypt_creds(creds_path: Option<&Path>, json: bool) -> Result<(), CryptoError> {
    let path = if let Some(p) = creds_path {
        p.to_path_buf()
    } else {
        let dir = cloudflared_dir();
        let mut found = None;
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            if entry
                .file_name()
                .to_string_lossy()
                .ends_with(".json.enc")
            {
                found = Some(entry.path());
                break;
            }
        }
        found.ok_or_else(|| {
            CryptoError::Other(format!("no .enc file found in {}", dir.display()))
        })?
    };

    let raw = fs::read(&path)?;
    let blob: EncryptedBlob =
        serde_json::from_slice(&raw).map_err(|e| CryptoError::Decrypt(e.to_string()))?;

    let key = get_or_create_key()?;
    let cipher = ChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;

    let nonce = Nonce::from_slice(&blob.nonce);
    let plaintext = cipher
        .decrypt(nonce, blob.ciphertext.as_ref())
        .map_err(|e| CryptoError::Decrypt(e.to_string()))?;

    let out_path = path.with_file_name(&blob.original_name);
    fs::write(&out_path, &plaintext)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&out_path, fs::Permissions::from_mode(0o400))?;
    }

    if json {
        println!(
            r#"{{"status":"decrypted","source":"{}","dest":"{}"}}"#,
            path.display(),
            out_path.display()
        );
    } else {
        println!("Decrypted: {} → {}", path.display(), out_path.display());
    }
    Ok(())
}
