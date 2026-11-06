//! Seals the model's reasoning so the page can carry it between requests without reading it.
//! DeepSeek needs earlier `reasoning_content` back whenever tools are in use.

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

const PREFIX: &str = "v1.";

pub struct Sealer {
    cipher: Aes256Gcm,
}

impl Sealer {
    pub fn new(key: &[u8; 32]) -> Self {
        Self { cipher: Aes256Gcm::new(key.into()) }
    }

    pub fn seal(&self, plain: &str) -> String {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ct = self.cipher.encrypt(&nonce, plain.as_bytes()).expect("AES-GCM encryption does not fail for in-memory input");
        let mut buf = nonce.to_vec();
        buf.extend_from_slice(&ct);
        format!("{PREFIX}{}", URL_SAFE_NO_PAD.encode(buf))
    }

    /// None when the blob is malformed, from another key, or was altered.
    pub fn open(&self, sealed: &str) -> Option<String> {
        let raw = URL_SAFE_NO_PAD.decode(sealed.strip_prefix(PREFIX)?).ok()?;
        if raw.len() < 12 + 16 {
            return None;
        }
        let (nonce, ct) = raw.split_at(12);
        let plain = self.cipher.decrypt(Nonce::from_slice(nonce), ct).ok()?;
        String::from_utf8(plain).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_tamper() {
        let s = Sealer::new(&[7u8; 32]);
        let blob = s.seal("thinking about shifts");
        assert!(blob.starts_with("v1."));
        assert!(!blob.contains("shifts"));
        assert_eq!(s.open(&blob).as_deref(), Some("thinking about shifts"));

        let mut bad = blob.clone().into_bytes();
        let last = bad.len() - 2;
        bad[last] = if bad[last] == b'A' { b'B' } else { b'A' };
        assert!(s.open(std::str::from_utf8(&bad).unwrap()).is_none());

        let other = Sealer::new(&[8u8; 32]);
        assert!(other.open(&blob).is_none());
        assert!(s.open("plain text").is_none());
    }
}
