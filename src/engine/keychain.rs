use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Default)]
struct KeychainVault {
    nonce_hex: String,
    ciphertext_hex: String,
}

pub struct EncryptedKeychain;

impl EncryptedKeychain {
    fn vault_path() -> PathBuf {
        dirs_fallback().join(".infinity_archiver_vault.json")
    }

    fn machine_key() -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"INFINITY_ARCHIVER_GAUNTLET_KEY_2026");
        if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
            hasher.update(user.as_bytes());
        }
        hasher.finalize().into()
    }

    pub fn save_passwords(passwords: &[String]) -> Result<()> {
        let key = Self::machine_key();
        let cipher = Aes256Gcm::new_from_slice(&key)?;
        let nonce_bytes: [u8; 12] = [0x49, 0x4E, 0x46, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39];
        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = serde_json::to_vec(passwords)?;
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Vault encryption failed: {}", e))?;

        let vault = KeychainVault {
            nonce_hex: hex_encode(&nonce_bytes),
            ciphertext_hex: hex_encode(&ciphertext),
        };
        fs::write(Self::vault_path(), serde_json::to_string_pretty(&vault)?)?;
        Ok(())
    }

    pub fn load_passwords() -> Vec<String> {
        let path = Self::vault_path();
        let Ok(raw) = fs::read_to_string(path) else {
            return vec!["infinity2026".into(), "admin".into()];
        };
        let Ok(vault) = serde_json::from_str::<KeychainVault>(&raw) else {
            return vec![];
        };
        let key = Self::machine_key();
        let Ok(cipher) = Aes256Gcm::new_from_slice(&key) else {
            return vec![];
        };
        let nonce_bytes = hex_decode(&vault.nonce_hex);
        let cipher_bytes = hex_decode(&vault.ciphertext_hex);
        if nonce_bytes.len() != 12 {
            return vec![];
        }
        let nonce = Nonce::from_slice(&nonce_bytes);
        if let Ok(plain) = cipher.decrypt(nonce, cipher_bytes.as_ref()) {
            serde_json::from_slice(&plain).unwrap_or_default()
        } else {
            vec![]
        }
    }
}

fn dirs_fallback() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| s.get(i..i + 2).and_then(|sub| u8::from_str_radix(sub, 16).ok()))
        .collect()
}

