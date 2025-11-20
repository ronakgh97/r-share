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
    let bytes =
        hex::decode(hex_str).map_err(|_e| Error::CryptoError(format!("Invalid hex public key")))?;

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
    hkdf.expand(b"File_Encryption_Test", &mut aes_key)
        .map_err(|_e| Error::CryptoError(format!("HKDF key derivation failed")))?;

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
