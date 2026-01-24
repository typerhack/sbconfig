// src/ssh/keys.rs
// SSH key generation

use crate::error::{AppError, Result};
use rand::rngs::OsRng;
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyType {
    Ed25519,
    Rsa,
}

impl KeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::Ed25519 => "ed25519",
            KeyType::Rsa => "rsa",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ed25519" => Some(KeyType::Ed25519),
            "rsa" | "rsa4096" => Some(KeyType::Rsa),
            _ => None,
        }
    }
}

pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
    pub key_type: KeyType,
}

/// Generate a new SSH key pair
pub fn generate_keypair(key_type: KeyType, comment: &str) -> Result<KeyPair> {
    let private_key = match key_type {
        KeyType::Ed25519 => PrivateKey::random(&mut OsRng, Algorithm::Ed25519)
            .map_err(|e| AppError::SshKey(e.to_string()))?,
        KeyType::Rsa => {
            // Use RSA with SHA-512 hash (default key size is 4096 bits)
            PrivateKey::random(
                &mut OsRng,
                Algorithm::Rsa {
                    hash: Some(HashAlg::Sha512),
                },
            )
            .map_err(|e| AppError::SshKey(e.to_string()))?
        }
    };

    // Get public key in OpenSSH format
    let public_key = private_key.public_key();
    let public_key_string = format!(
        "{} {}",
        public_key
            .to_openssh()
            .map_err(|e| AppError::SshKey(e.to_string()))?,
        comment
    );

    // Get private key in OpenSSH format
    let private_key_string = private_key
        .to_openssh(LineEnding::LF)
        .map_err(|e| AppError::SshKey(e.to_string()))?
        .to_string();

    Ok(KeyPair {
        public_key: public_key_string,
        private_key: private_key_string,
        key_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ed25519() {
        let keypair = generate_keypair(KeyType::Ed25519, "test@sbconfig").unwrap();
        assert!(keypair.public_key.starts_with("ssh-ed25519 "));
        assert!(keypair.public_key.ends_with(" test@sbconfig"));
        assert!(keypair.private_key.contains("OPENSSH PRIVATE KEY"));
    }

    #[test]
    fn test_generate_rsa() {
        let keypair = generate_keypair(KeyType::Rsa, "test@sbconfig").unwrap();
        assert!(keypair.public_key.starts_with("ssh-rsa "));
        assert!(keypair.private_key.contains("OPENSSH PRIVATE KEY"));
    }
}
