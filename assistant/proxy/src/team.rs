//! Team passphrase: lets the team use the assistant during quiet hours.
//! Only an Argon2id hash is stored, base64-encoded so the `$` signs survive env files.

use anyhow::{Context, Result, bail};
use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Mutex;

pub const MIN_LEN: usize = 16;

pub fn hash_for_env(passphrase: &str) -> Result<String> {
    if passphrase.chars().count() < MIN_LEN {
        bail!("use at least {MIN_LEN} characters; five random words work well");
    }
    let salt = SaltString::generate(&mut OsRng);
    let phc = Argon2::default().hash_password(passphrase.as_bytes(), &salt).map_err(|e| anyhow::anyhow!("hashing failed: {e}"))?.to_string();
    Ok(STANDARD.encode(phc))
}

pub fn decode_stored_hash(b64: &str) -> Result<String> {
    let bytes = STANDARD.decode(b64).context("TEAM_PASS_HASH is not base64 (create it with `jobex-assistant hash-pass`)")?;
    let phc = String::from_utf8(bytes).context("TEAM_PASS_HASH is not text")?;
    PasswordHash::new(&phc).map_err(|e| anyhow::anyhow!("TEAM_PASS_HASH is not a valid hash: {e}"))?;
    Ok(phc)
}

/// Verifies passphrases and remembers ones that already passed, so Argon2 runs once per passphrase.
pub struct TeamGate {
    phc: Option<String>,
    accepted: Mutex<HashSet<[u8; 32]>>,
}

impl TeamGate {
    pub fn new(phc: Option<String>) -> Self {
        Self { phc, accepted: Mutex::new(HashSet::new()) }
    }

    pub fn configured(&self) -> bool {
        self.phc.is_some()
    }

    pub fn check(&self, candidate: &str) -> bool {
        let Some(phc) = &self.phc else { return false };
        let digest: [u8; 32] = Sha256::digest(candidate.as_bytes()).into();
        if self.accepted.lock().unwrap().contains(&digest) {
            return true;
        }
        let ok = PasswordHash::new(phc).map(|h| Argon2::default().verify_password(candidate.as_bytes(), &h).is_ok()).unwrap_or(false);
        if ok {
            self.accepted.lock().unwrap().insert(digest);
        }
        ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_decode_verify() {
        assert!(hash_for_env("short").is_err());
        let stored = hash_for_env("orange tram violin quiet harbor").unwrap();
        assert!(!stored.contains('$'));
        let gate = TeamGate::new(Some(decode_stored_hash(&stored).unwrap()));
        assert!(gate.check("orange tram violin quiet harbor"));
        assert!(gate.check("orange tram violin quiet harbor"));
        assert!(!gate.check("orange tram violin quiet harbour"));
        assert!(!TeamGate::new(None).check("anything at all here"));
    }
}
