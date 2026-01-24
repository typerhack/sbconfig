// src/db/encryption.rs
// Private key encryption/decryption using AES-256-GCM
#![allow(dead_code)]

use crate::error::{AppError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// Derive a 256-bit key from a password using simple hashing
/// In production, you might want to use a proper KDF like Argon2
fn derive_key(password: &str) -> [u8; KEY_SIZE] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut key = [0u8; KEY_SIZE];
    let mut hasher = DefaultHasher::new();

    // Hash the password multiple times to fill the key
    for i in 0..4 {
        password.hash(&mut hasher);
        i.hash(&mut hasher);
        let hash = hasher.finish().to_le_bytes();
        key[i * 8..(i + 1) * 8].copy_from_slice(&hash);
    }

    key
}

/// Encrypt data with AES-256-GCM
pub fn encrypt(plaintext: &str, password: &str) -> Result<String> {
    let key = derive_key(password);
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| AppError::Encryption(e.to_string()))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Encryption(e.to_string()))?;

    // Combine nonce + ciphertext and encode as base64
    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(encrypted: &str, password: &str) -> Result<String> {
    let key = derive_key(password);
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| AppError::Encryption(e.to_string()))?;

    // Decode base64
    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| AppError::Encryption(e.to_string()))?;

    if combined.len() < NONCE_SIZE {
        return Err(AppError::Encryption("Invalid encrypted data".to_string()));
    }

    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| AppError::Encryption(e.to_string()))?;

    String::from_utf8(plaintext).map_err(|e| AppError::Encryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "This is a secret SSH private key!";
        let password = "my_secure_password";

        let encrypted = encrypt(plaintext, password).unwrap();
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt(&encrypted, password).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password() {
        let plaintext = "Secret data";
        let encrypted = encrypt(plaintext, "correct_password").unwrap();

        let result = decrypt(&encrypted, "wrong_password");
        assert!(result.is_err());
    }
}
