use crate::utils::error::{Error, Result};
use aes_gcm::aead::OsRng;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};

/// Encrypt a chunk of data using AES-256-GCM
///
/// Returns: [12-byte nonce][ciphertext][16-byte authentication tag]
/// The nonce and tag are prepended/appended so the receiver can decrypt without out-of-band metadata exchange.
pub fn encrypt_chunk(aes_key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>> {
    // Create cipher from key
    let cipher = Aes256Gcm::new(aes_key.into());

    // Generate random 96-bit nonce (12 bytes)
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    #[allow(deprecated)]
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt and authenticate
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| Error::CryptoError(format!("AES-GCM encryption failed: {}", e)))?;

    // Format: [nonce (12 bytes)][ciphertext + tag (N + 16 bytes)]
    let mut encrypted = Vec::with_capacity(12 + ciphertext.len());
    encrypted.extend_from_slice(&nonce_bytes);
    encrypted.extend_from_slice(&ciphertext);

    Ok(encrypted)
}

/// Decrypt a chunk of data using AES-256-GCM
///
/// Expects input format: [12-byte nonce][ciphertext][16-byte authentication tag]
/// Returns the original plaintext if authentication succeeds.
pub fn decrypt_chunk(aes_key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>> {
    // Validate minimum length (nonce + tag)
    if encrypted.len() < 12 + 16 {
        return Err(Error::CryptoError(format!(
            "Encrypted data too short: {} bytes (need at least 28)",
            encrypted.len()
        )));
    }

    // Extract nonce (first 12 bytes)
    #[allow(deprecated)]
    let nonce = Nonce::from_slice(&encrypted[..12]);

    // Extract ciphertext + tag (remaining bytes)
    let ciphertext = &encrypted[12..];

    // Create cipher from key
    let cipher = Aes256Gcm::new(aes_key.into());

    // Decrypt and verify authentication tag
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_e| {
        Error::CryptoError(format!(
            "Decryption failed (authentication failure or corrupted data)",
        ))
    })?;

    Ok(plaintext)
}

/// Encrypt an entire file in chunks and return encrypted bytes
/// This is useful for small files that fit in memory
#[allow(dead_code)]
pub fn encrypt_file(aes_key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>> {
    encrypt_chunk(aes_key, data)
}

/// Decrypt an entire file
#[allow(dead_code)]
pub fn decrypt_file(aes_key: &[u8; 32], encrypted_data: &[u8]) -> Result<Vec<u8>> {
    decrypt_chunk(aes_key, encrypted_data)
}
