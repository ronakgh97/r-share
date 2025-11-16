use crate::config::constants::*;
use crate::crypto;
use crate::crypto::{encryption, key_exchange, signing};
use crate::dirs::{config, contacts, keys};
use crate::server::RelayClient;
use crate::utils::error::{Error, Result};
use crate::utils::hash;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
//use memmap2::MmapMut;
//use std::fs::OpenOptions;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

/// Listen for incoming file transfers
pub async fn run(path: Option<PathBuf>, from: String, _quiet: bool, local: bool) -> Result<()> {
    println!("{}", "Listening...\n".bright_green().bold());

    // Load config and keys
    let config = config::load_config()?;
    let (_signing_key, verifying_key) = keys::load_keys_from(&config.path.keys_path)?;
    let my_fingerprint = hex::encode(verifying_key.to_bytes());

    // Determine download path
    let download_path = path.unwrap_or_else(|| config.path.download_path.clone());
    std::fs::create_dir_all(&download_path)?;

    //println!("{} Ready to receive files", "✓".bright_green());
    //println!(
    //    "   Save to: {}",
    //    download_path.display().to_string().bright_yellow()
    //);
    //println!(
    //    "   Fingerprint: {}...",
    //    &my_fingerprint[..16].bright_cyan().dimmed()
    //);

    // Load contacts for verification
    let contact_list = contacts::load_contacts()?;

    // Find expected sender
    let expected_sender = contact_list.get(&from).ok_or_else(|| {
        Error::InvalidInput(format!("Contact '{}' not found in trusted contacts", from))
    })?;

    //println!();

    // Create relay client
    let relay_client = if local {
        RelayClient::new(
            config.server.private_ip.clone(),
            DEFAULT_HTTP_PORT.to_string().clone(),
            DEFAULT_SOCKET_PORT.clone(),
        )
    } else {
        RelayClient::new(
            config.server.public_ip.clone(),
            DEFAULT_HTTP_PORT.to_string().clone(),
            DEFAULT_SOCKET_PORT.clone(),
        )
    };

    // Generate ephemeral X25519 keypair for this transfer
    //println!(
    //    "{}",
    //" Generating ephemeral encryption keys...".bright_cyan()
    //);

    let ephemeral_keypair = crypto::key_exchange::EphemeralKeyPair::generate();
    let receiver_ephemeral_hex = ephemeral_keypair.public_key_hex();

    //println!(
    //    "{}  Ephemeral key: {}...",
    //    "✓".bright_green(),
    //    &receiver_ephemeral_hex[..16].bright_cyan().dimmed()
    //);
    //println!();

    // Join transfer session (blocks until sender connects)
    println!("{}", "Waiting for sender to connect...".yellow());
    let mut session = relay_client
        .listen(my_fingerprint.clone(), receiver_ephemeral_hex)
        .await?;

    println!("  Session: {}", session.session_id().bright_green());
    //println!();

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

    //println!("{} Signature verified", "✓".bright_green());
    //println!(
    //    "   Expected hash: {}...",
    //    &file_hash_from_sender[..16].bright_cyan().dimmed()
    //);

    // Derive encryption key from ephemeral keys
    let sender_ephemeral_hex = session
        .sender_ephemeral_key
        .as_ref()
        .ok_or_else(|| Error::CryptoError("Sender ephemeral key not found".into()))?;

    //println!("{}", " Deriving encryption key...".bright_cyan());
    let aes_key = key_exchange::perform_key_exchange(
        ephemeral_keypair.secret,
        sender_ephemeral_hex,
        session.session_id(),
    )?;
    //println!("{}  Encryption key derived", "✓".bright_green());

    //println!("{} Incoming file transfer", "✓".bright_green());
    println!();
    println!(
        " File: {} | Hash {}...",
        filename.bright_yellow(),
        &file_hash_from_sender[..16].bright_cyan().dimmed()
    );
    println!(
        " Size: {} bytes ({:.2} MB)",
        filesize,
        filesize as f64 / (1024.0 * 1024.0)
    );
    println!(" From:   {}", from.bright_white().bold());
    println!();
    println!(
        "{} Receiving and decrypting file...",
        "⇙".bright_magenta().bold()
    );

    // Receive encrypted file data with progress bar
    let file_path = download_path.join(&filename);
    let file_writer = File::create(&file_path).await?;
    let mut file_writer = BufWriter::with_capacity(BUFFER_SIZE, file_writer);
    //let file = OpenOptions::new()
    //    .read(true)
    //    .write(true)
    //    .create(true)
    //    .open(&file_path)?;
    //file.set_len(filesize)?;
    //let mut mmap = unsafe { MmapMut::map_mut(&file)? };

    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(PROGRESS_BAR_TEMPLATE)
            .unwrap()
            .progress_chars(PROGRESS_BAR_CHARS)
            .tick_chars(DEFAULT_SPINNER_STYLE),
    );

    let mut total_received = 0u64;
    //let mut offset = 0;

    // Read encrypted chunks: [4B size][encrypted data]
    while total_received < filesize {
        // Read 4-byte size prefix
        let mut size_buffer = [0u8; 4];
        if let Err(e) = session.read_exact(&mut size_buffer).await {
            println!();
            println!(
                "{} Connection closed early! Received {}/{} bytes ({:.1}%)",
                "✗".bright_red().bold(),
                total_received,
                filesize,
                (total_received as f64 / filesize as f64) * 100.0
            );

            // Clean up partial file immediately
            //drop(mmap);
            //std::fs::remove_file(&file_path)?;
            drop(file_writer);
            tokio::fs::remove_file(&file_path).await?;
            println!("{} Partial file deleted", "✓".bright_red());

            return Err(Error::InvalidInput(format!(
                "Transfer interrupted - connection closed before size prefix: {}",
                e
            )));
        }

        let chunk_size = u32::from_be_bytes(size_buffer) as usize;

        // Read encrypted chunk
        let mut encrypted_buffer = vec![0u8; chunk_size];
        if let Err(e) = session.read_exact(&mut encrypted_buffer).await {
            println!();
            println!(
                "{} Connection closed early! Received {}/{} bytes ({:.1}%)",
                "✗".bright_red().bold(),
                total_received,
                filesize,
                (total_received as f64 / filesize as f64) * 100.0
            );

            // Clean up partial file immediately
            //drop(mmap);
            //std::fs::remove_file(&file_path)?;
            drop(file_writer);
            tokio::fs::remove_file(&file_path).await?;
            println!("{} Partial file deleted", "✓".bright_red());

            return Err(Error::InvalidInput(format!(
                "Transfer interrupted - connection closed during chunk: {}",
                e
            )));
        }

        // Decrypt the chunk
        let plaintext = encryption::decrypt_chunk(&aes_key, &encrypted_buffer)?;

        // Write decrypted data to file
        //let len = plaintext.len();
        //mmap[offset..offset + len].copy_from_slice(&plaintext);
        file_writer.write_all(&plaintext).await?;
        total_received += plaintext.len() as u64;
        //offset += len;
        //total_received += len as u64;

        pb.set_position(total_received);
    }

    //mmap.flush()?;
    file_writer.flush().await?;
    pb.finish_with_message("Download complete!");

    // Verify file integrity by computing SHA256 hash
    println!();
    println!();
    println!("{}", "Verifying file hash...".yellow());

    // Use the extracted file_io utility
    let computed_hash = hash::compute_file_hash(&file_path).await?;

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
        //std::fs::remove_file(&file_path)?;
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

    println!(
        "  File hash verified | Hash {}...",
        &computed_hash[..16].bright_cyan().dimmed()
    );
    //println!(
    //    "   Hash: {}...",
    //    &computed_hash[..KEY_FINGERPRINT_DISPLAY_LEN]
    //        .bright_cyan()
    //         .dimmed()
    //);

    // Send completion confirmation to sender
    //println!();
    //println!(" Sending completion signal to sender...");
    session.write_all(DONE_SIGNAL).await?;
    session.flush().await?;

    println!();
    println!("{} File received successfully!", "✓".bright_green().bold());

    //println!("   Saved to: {}", file_path.display());
    //println!(
    //   "   Size: {} bytes ({:.2} MB)",
    //    total_received,
    //    total_received as f64 / (1024.0 * 1024.0)
    //);

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
