// aura-silo/src/lib.rs

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SiloError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Encryption failed")]
    EncryptionFailed,
}

pub struct Cookie {
    pub host: String,
    pub name: String,
    pub value: Vec<u8>,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
    pub expiry_utc: Option<i64>,
}

pub struct SiloManager {
    base_dir: PathBuf,
    master_key: [u8; 32],
}

impl SiloManager {
    pub fn init(base_dir: PathBuf) -> Result<Self, SiloError> {
        let entry = keyring::Entry::new("aura-browser", "master-key")
            .map_err(|_| SiloError::EncryptionFailed)?;

        let master_key = match entry.get_password() {
            Ok(pw) => {
                let decoded = hex::decode(pw).map_err(|_| SiloError::EncryptionFailed)?;
                let mut key = [0u8; 32];
                key.copy_from_slice(&decoded);
                key
            }
            Err(_) => {
                // Generate new master key
                let mut key = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut key);
                let encoded = hex::encode(key);
                entry
                    .set_password(&encoded)
                    .map_err(|_| SiloError::EncryptionFailed)?;
                key
            }
        };

        Ok(Self {
            base_dir,
            master_key,
        })
    }

    /// Derive per-domain silo path
    fn silo_path(&self, registrable_domain: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(registrable_domain.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.base_dir.join(format!("{}.silo.db", hash))
    }

    /// Open (or create) a silo for a given domain
    pub fn open_silo(&self, registrable_domain: &str) -> Result<Connection, SiloError> {
        let path = self.silo_path(registrable_domain);
        let conn = Connection::open(&path)?;

        // Apply WAL + schema migrations
        conn.execute_batch(include_str!("schema.sql"))?;

        // Register domain in meta if new
        conn.execute(
            "INSERT OR IGNORE INTO silo_meta VALUES ('domain', ?)",
            params![registrable_domain],
        )?;

        Ok(conn)
    }

    /// Set a cookie (encrypts value before writing)
    pub fn set_cookie(&self, domain: &str, cookie: &Cookie) -> Result<(), SiloError> {
        let conn = self.open_silo(domain)?;
        let encrypted_value = self.encrypt_value(&cookie.value)?;

        conn.execute(
            "INSERT OR REPLACE INTO cookies
             (host, name, value, path, secure, http_only, same_site, expiry_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                cookie.host,
                cookie.name,
                encrypted_value,
                cookie.path,
                cookie.secure as i32,
                cookie.http_only as i32,
                cookie.same_site.as_str(),
                cookie.expiry_utc,
            ],
        )?;
        Ok(())
    }

    /// Purge all non-pinned silos (called on session close)
    pub fn purge_session_silos(&self) -> Result<usize, SiloError> {
        let mut purged = 0usize;
        for entry in std::fs::read_dir(&self.base_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |e| e == "db") {
                // Check pinned status
                if let Ok(conn) = Connection::open(&path) {
                    let pinned: i32 = conn
                        .query_row("SELECT value FROM silo_meta WHERE key='pinned'", [], |r| {
                            r.get(0)
                        })
                        .unwrap_or(0);

                    if pinned == 0 {
                        drop(conn);
                        std::fs::remove_file(&path)?;
                        purged += 1;
                    }
                }
            }
        }
        Ok(purged)
    }

    fn encrypt_value(&self, plaintext: &[u8]) -> Result<Vec<u8>, SiloError> {
        let key = Key::from_slice(&self.master_key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| SiloError::EncryptionFailed)?;

        // Prepend nonce so we can decrypt later: [12-byte nonce][ciphertext]
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
}
