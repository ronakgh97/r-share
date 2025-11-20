use crate::utils::error::{Error, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
// Verifier trait is required for verify_strict method
#[allow(unused_imports)]
use ed25519_dalek::Verifier;

/// Sign data and return signature
pub fn sign_data(signing_key: &SigningKey, data: &str) -> Result<Signature> {
    Ok(signing_key.sign(data.as_bytes()))
}

/// Verify signature
pub fn verify_signature(
    verifying_key: &VerifyingKey,
    data: &str,
    signature: &Signature,
) -> Result<()> {
    verifying_key
        .verify_strict(data.as_bytes(), signature)
        .map_err(|_e| Error::InvalidInput(format!("Signature verification failed")))
}
