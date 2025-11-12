use rshare::crypto::encryption::{decrypt_chunk, encrypt_chunk};

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
