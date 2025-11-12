use rshare::crypto::key_exchange::{
    EphemeralKeyPair, compute_shared_secret, derive_aes_key, parse_public_key, perform_key_exchange,
};

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
