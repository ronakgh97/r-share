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
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        Error::CryptoError(format!(
            "AES-GCM decryption failed (authentication failure or corrupted data): {}",
            e
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_chunk() {
        let aes_key = [0x42u8; 32];
        let plaintext = b"Hello, secure world!";

        // Encrypt
        let encrypted = encrypt_chunk(&aes_key, plaintext).unwrap();

        // Should have nonce (12) + plaintext length + tag (16)
        assert_eq!(encrypted.len(), 12 + plaintext.len() + 16);

        // Decrypt
        let decrypted = decrypt_chunk(&aes_key, &encrypted).unwrap();

        // Should match original
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let correct_key = [0x42u8; 32];
        let wrong_key = [0x99u8; 32];
        let plaintext = b"Secret message";

        // Encrypt with correct key
        let encrypted = encrypt_chunk(&correct_key, plaintext).unwrap();

        // Decrypt with wrong key should fail
        let result = decrypt_chunk(&wrong_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_corrupted_data() {
        let aes_key = [0x42u8; 32];
        let plaintext = b"Test data";

        // Encrypt
        let mut encrypted = encrypt_chunk(&aes_key, plaintext).unwrap();

        // Corrupt one byte in the middle
        encrypted[20] ^= 0xFF;

        // Decryption should fail due to authentication tag mismatch
        let result = decrypt_chunk(&aes_key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_truncated_data() {
        let aes_key = [0x42u8; 32];

        // Data too short (less than nonce + tag)
        let short_data = vec![0u8; 20];

        let result = decrypt_chunk(&aes_key, &short_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_data() {
        let aes_key = [0x42u8; 32];
        let plaintext = b"";

        let encrypted = encrypt_chunk(&aes_key, plaintext).unwrap();

        // Should still have nonce (12) + tag (16)
        assert_eq!(encrypted.len(), 28);

        let decrypted = decrypt_chunk(&aes_key, &encrypted).unwrap();
        assert_eq!(decrypted.len(), 0);
    }

    #[test]
    fn test_different_nonces() {
        let aes_key = [0x42u8; 32];
        let plaintext = b"Same message";

        // Encrypt twice
        let encrypted1 = encrypt_chunk(&aes_key, plaintext).unwrap();
        let encrypted2 = encrypt_chunk(&aes_key, plaintext).unwrap();

        // Ciphertexts should differ (different random nonces)
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt correctly
        let decrypted1 = decrypt_chunk(&aes_key, &encrypted1).unwrap();
        let decrypted2 = decrypt_chunk(&aes_key, &encrypted2).unwrap();

        assert_eq!(decrypted1, plaintext);
        assert_eq!(decrypted2, plaintext);
    }

    #[test]
    fn test_large_chunk() {
        let aes_key = [0x42u8; 32];
        let plaintext = vec![0x55u8; 1024 * 1024]; // 1 MB

        let encrypted = encrypt_chunk(&aes_key, &plaintext).unwrap();
        let decrypted = decrypt_chunk(&aes_key, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
