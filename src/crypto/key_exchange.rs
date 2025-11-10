use crate::utils::error::{Error, Result};
use aes_gcm::aead::rand_core::OsRng;
use hkdf::Hkdf;
use sha2::Sha256;
use x25519_dalek::{EphemeralSecret, PublicKey};

/// Ephemeral keypair for a single file transfer session
pub struct EphemeralKeyPair {
    pub secret: EphemeralSecret,
    pub public: PublicKey,
}

impl EphemeralKeyPair {
    /// Generate a fresh ephemeral X25519 keypair for this transfer
    /// This keypair is temporary and should be deleted after the transfer completes.
    pub fn generate() -> Self {
        let secret = EphemeralSecret::random_from_rng(&mut OsRng);
        let public = PublicKey::from(&secret);

        EphemeralKeyPair { secret, public }
    }

    /// Get the public key as a hex-encoded string for transmission
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public.as_bytes())
    }
}

/// Parse a hex-encoded X25519 public key
pub fn parse_public_key(hex_str: &str) -> Result<PublicKey> {
    let bytes = hex::decode(hex_str)
        .map_err(|e| Error::CryptoError(format!("Invalid hex public key: {}", e)))?;

    if bytes.len() != 32 {
        return Err(Error::CryptoError(format!(
            "Invalid public key length: expected 32 bytes, got {}",
            bytes.len()
        )));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&bytes);

    Ok(PublicKey::from(key_bytes))
}

/// Compute the ECDH shared secret from our ephemeral private key and their public key
pub fn compute_shared_secret(our_secret: EphemeralSecret, their_public: PublicKey) -> [u8; 32] {
    let shared_secret = our_secret.diffie_hellman(&their_public);
    *shared_secret.as_bytes()
}

/// Derive a 256-bit AES key from the shared secret using HKDF-SHA256
///
/// Uses the session_id as salt to ensure unique keys even if ephemeral keys are reused
pub fn derive_aes_key(shared_secret: &[u8; 32], session_id: &str) -> Result<[u8; 32]> {
    let hkdf = Hkdf::<Sha256>::new(Some(session_id.as_bytes()), shared_secret);

    let mut aes_key = [0u8; 32]; // 256 bits for AES-256
    hkdf.expand(b"rshare-file-encryption-v1", &mut aes_key)
        .map_err(|e| Error::CryptoError(format!("HKDF key derivation failed: {}", e)))?;

    Ok(aes_key)
}

/// Complete key exchange: compute shared secret and derive AES key
pub fn perform_key_exchange(
    our_secret: EphemeralSecret,
    their_public_hex: &str,
    session_id: &str,
) -> Result<[u8; 32]> {
    // Parse their public key
    let their_public = parse_public_key(their_public_hex)?;

    // Compute ECDH shared secret
    let shared_secret = compute_shared_secret(our_secret, their_public);

    // Derive AES key using HKDF
    derive_aes_key(&shared_secret, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_keypair_generation() {
        let keypair = EphemeralKeyPair::generate();
        let hex = keypair.public_key_hex();

        // Should be 32 bytes = 64 hex chars
        assert_eq!(hex.len(), 64);

        // Should be valid hex
        assert!(hex::decode(&hex).is_ok());
    }

    #[test]
    fn test_parse_public_key() {
        let keypair = EphemeralKeyPair::generate();
        let hex = keypair.public_key_hex();

        let parsed = parse_public_key(&hex).unwrap();
        assert_eq!(parsed.as_bytes(), keypair.public.as_bytes());
    }

    #[test]
    fn test_key_exchange_symmetric() {
        // Alice generates keypair
        let alice_keypair = EphemeralKeyPair::generate();

        // Bob generates keypair
        let bob_keypair = EphemeralKeyPair::generate();

        // Alice computes shared secret with Bob's public key
        let alice_shared = compute_shared_secret(alice_keypair.secret, bob_keypair.public);

        // Bob computes shared secret with Alice's public key
        let bob_shared = compute_shared_secret(bob_keypair.secret, alice_keypair.public);

        // Shared secrets should match (ECDH property)
        assert_eq!(alice_shared, bob_shared);
    }

    #[test]
    fn test_derive_aes_key() {
        let shared_secret = [0x42u8; 32];
        let session_id = "test-session-123";

        let key = derive_aes_key(&shared_secret, session_id).unwrap();

        // Should produce 32-byte key
        assert_eq!(key.len(), 32);

        // Different session IDs should produce different keys
        let key2 = derive_aes_key(&shared_secret, "different-session").unwrap();
        assert_ne!(key, key2);
    }

    #[test]
    fn test_full_key_exchange() {
        let alice = EphemeralKeyPair::generate();
        let bob = EphemeralKeyPair::generate();
        let session_id = "test-session-456";

        let alice_public_hex = alice.public_key_hex();
        let bob_public_hex = bob.public_key_hex();

        // Alice derives AES key
        let alice_aes = perform_key_exchange(alice.secret, &bob_public_hex, session_id).unwrap();

        // Bob derives AES key
        let bob_aes = perform_key_exchange(bob.secret, &alice_public_hex, session_id).unwrap();

        // Both should have the same AES key
        assert_eq!(alice_aes, bob_aes);
    }
}
