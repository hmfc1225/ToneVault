use rand::Rng;
use sha2::{Digest, Sha256};

pub struct ApiKeyGenerator;

impl ApiKeyGenerator {
    pub fn generate() -> (String, String, String) {
        let raw = Self::generate_raw_key();
        let prefix = raw[..8].to_string();
        let hash = Self::hash_key(&raw);
        (raw, prefix, hash)
    }

    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_key(key: &str, hash: &str) -> bool {
        Self::hash_key(key) == hash
    }

    fn generate_raw_key() -> String {
        let prefix = "tv_";
        let random: [u8; 32] = rand::thread_rng().gen();
        let encoded = hex::encode(random);
        format!("{}{}", prefix, encoded)
    }
}

mod hex {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

    pub fn encode(bytes: [u8; 32]) -> String {
        let mut s = String::with_capacity(64);
        for &b in &bytes {
            s.push(HEX_CHARS[(b >> 4) as usize] as char);
            s.push(HEX_CHARS[(b & 0x0f) as usize] as char);
        }
        s
    }
}
