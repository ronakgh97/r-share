use crate::crypto::signing;
use crate::dirs::{config, contacts, keys};
use crate::server::RelayClient;
use crate::utils::error::{Error, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Listen for incoming file transfers
pub async fn run(path: Option<PathBuf>, from: String, _quiet: bool) -> Result<()> {
    println!("{}", "Listening...\n".bright_cyan().bold());

    // Load config and keys
    let config = config::load_config()?;
    let (_signing_key, verifying_key) = keys::load_keys_from(&config.path.keys_path)?;
    let my_fingerprint = hex::encode(verifying_key.to_bytes());

    // Determine download path
    let download_path = path.unwrap_or_else(|| config.path.download_path.clone());
    std::fs::create_dir_all(&download_path)?;

    println!("{} Ready to receive files", "✓".bright_green());
    println!(
        "   Save to: {}",
        download_path.display().to_string().bright_yellow()
    );
    println!(
        "   Fingerprint: {}...",
        &my_fingerprint[..16].bright_cyan().dimmed()
    );

    // Load contacts for verification
    let contact_list = contacts::load_contacts()?;

    // Find expected sender
    let expected_sender = contact_list.get(&from).ok_or_else(|| {
        Error::InvalidInput(format!("Contact '{}' not found in trusted contacts", from))
    })?;

    println!();

    // Create relay client from config
    let relay_client = RelayClient::new(
        config.server.http_url.clone(),
        config.server.socket_host.clone(),
        config.server.socket_port,
    );

    // Join transfer session (blocks until sender connects)
    println!("{}", " Waiting for sender to connect...".yellow());
    let mut session = relay_client.listen(my_fingerprint.clone()).await?;

    println!(
        "{} Sender connected! Session: {}",
        "✓".bright_green(),
        session.session_id().bright_cyan()
    );
    println!();

    // Extract metadata from HTTP response
    let filename = session
        .filename
        .clone()
        .ok_or_else(|| Error::InvalidInput("No filename in session".into()))?;
    let filesize = session
        .file_size
        .ok_or_else(|| Error::InvalidInput("No file size in session".into()))?;
    let signature_hex = session
        .signature
        .clone()
        .ok_or_else(|| Error::InvalidInput("No signature in session".into()))?;
    let sender_fp = session
        .sender_fp
        .clone()
        .ok_or_else(|| Error::InvalidInput("No sender fingerprint in session".into()))?;
    let file_hash_from_sender = session
        .file_hash
        .clone()
        .ok_or_else(|| Error::InvalidInput("No file hash in session".into()))?;

    // Verify sender is the expected contact
    if expected_sender.public_key != sender_fp {
        return Err(Error::InvalidInput(format!(
            "Sender fingerprint mismatch! Expected {}, got {}",
            &expected_sender.public_key[..16],
            &sender_fp[..16]
        )));
    }

    // Decode sender's public key for signature verification
    let sender_key_bytes = hex::decode(&sender_fp)
        .map_err(|_| Error::InvalidInput("Invalid sender public key".into()))?;
    let sender_key = ed25519_dalek::VerifyingKey::from_bytes(
        sender_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidInput("Invalid key length".into()))?,
    )
    .map_err(|_| Error::InvalidInput("Invalid sender key".into()))?;

    // Verify Ed25519 signature on metadata (filename|filesize|hash)
    let metadata_msg = format!("{}|{}|{}", filename, filesize, file_hash_from_sender);
    let signature_bytes = hex::decode(&signature_hex)
        .map_err(|_| Error::InvalidInput("Invalid signature hex".into()))?;
    let signature = ed25519_dalek::Signature::from_bytes(
        signature_bytes
            .as_slice()
            .try_into()
            .map_err(|_| Error::InvalidInput("Invalid signature length".into()))?,
    );

    // Verify signature
    if let Err(_) = signing::verify_signature(&sender_key, &metadata_msg, &signature) {
        println!();
        println!("{} SIGNATURE VERIFICATION FAILED!", "✗".bright_red().bold());
        println!("   Sender claims: {}...", &sender_fp[..16].bright_red());
        if let Some(ref expected) = expected_sender.public_key.get(..16) {
            println!("   Expected from: {}...", expected.bright_yellow());
        }
        println!();
        println!(
            "{} This could be an impersonation attempt or corrupted metadata!",
            "✗".bright_yellow()
        );
        println!("{} Transfer REJECTED.", "✗".bright_red().bold());

        // Send error signal to sender
        let _ = session.write_all(b"ERROR:signature_failed\n").await;
        let _ = session.flush().await;

        return Err(Error::InvalidInput("Signature verification failed".into()));
    }

    println!("{} Signature verified", "✓".bright_green());
    println!(
        "   Expected hash: {}...",
        &file_hash_from_sender[..16].bright_cyan().dimmed()
    );

    println!("{} Incoming file transfer", "✓".bright_green());
    println!("   File: {}", filename.bright_yellow());
    println!(
        "   Size: {} bytes ({:.2} MB)",
        filesize,
        filesize as f64 / (1024.0 * 1024.0)
    );
    println!();
    println!("{} Receiving file...", "◆".bright_cyan());

    // Receive file data with progress bar
    let file_path = download_path.join(&filename);
    let mut file_writer = File::create(&file_path).await?;

    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [ {bar:60.cyan/blue} ] {bytes}/{total_bytes} ({bytes_per_sec}) ⏱ ({eta})")
            .unwrap()
            .progress_chars("░▒▓█"),
    );

    let mut buffer = vec![0u8; 64 * 1024]; // 64KB chunks
    let mut total_received = 0u64;

    while total_received < filesize {
        let n = session.read(&mut buffer).await?;
        if n == 0 {
            println!();
            println!(
                "{} Connection closed early! Received {}/{} bytes ({:.1}%)",
                "✗".bright_red().bold(),
                total_received,
                filesize,
                (total_received as f64 / filesize as f64) * 100.0
            );

            // Clean up partial file immediately
            drop(file_writer);
            tokio::fs::remove_file(&file_path).await?;
            println!("{} Partial file deleted", "✓".bright_red());

            // Note: Cannot send ERROR signal here as connection is already closed

            return Err(Error::InvalidInput(
                "Transfer interrupted - connection closed early".into(),
            ));
        }

        file_writer.write_all(&buffer[..n]).await?;
        total_received += n as u64;
        pb.set_position(total_received);
    }

    file_writer.flush().await?;
    pb.finish_with_message("Download complete!");

    // Verify file integrity by computing SHA256 hash
    println!();
    println!("{}", " Verifying file integrity...".bright_cyan());

    let mut hasher = Sha256::new();
    let mut file_for_hash = File::open(&file_path).await?;
    let mut hash_buffer = vec![0u8; 64 * 1024];

    loop {
        let n = file_for_hash.read(&mut hash_buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&hash_buffer[..n]);
    }

    let computed_hash = hex::encode(hasher.finalize());

    // Compare with expected hash from signature
    if computed_hash != file_hash_from_sender {
        println!();
        println!("{} FILE INTEGRITY CHECK FAILED!", "✗".bright_red().bold());
        println!(
            "   Expected: {}...",
            &file_hash_from_sender[..16].bright_yellow()
        );
        println!("   Got:      {}...", &computed_hash[..16].bright_red());
        println!();

        // Delete corrupted file
        tokio::fs::remove_file(&file_path).await?;
        println!("{} Corrupted file deleted: {}", "✓".bright_red(), filename);
        println!(
            "{} The file may have been tampered with or corrupted during transfer!",
            "✗".bright_yellow()
        );

        // Send error signal to sender
        let _ = session.write_all(b"ERROR:hash_mismatch\n").await;
        let _ = session.flush().await;

        return Err(Error::InvalidInput("File integrity check failed".into()));
    }

    println!("{} File integrity verified", "✓".bright_green());
    println!(
        "   Hash: {}...",
        &computed_hash[..16].bright_cyan().dimmed()
    );

    // Send completion confirmation to sender
    println!();
    println!(" Sending completion signal to sender...");
    session.write_all(b"DONE\n").await?;
    session.flush().await?;

    println!();
    println!("{} File received successfully! ;)", "✓".bright_green().bold());
    println!("   Saved to: {}", file_path.display());
    println!(
        "   Size: {} bytes ({:.2} MB)",
        total_received,
        total_received as f64 / (1024.0 * 1024.0)
    );

    if total_received < filesize {
        println!(
            "   {} Expected {} bytes, got {} bytes",
            "✗".bright_yellow().bold(),
            filesize,
            total_received
        );
    }

    Ok(())
}
