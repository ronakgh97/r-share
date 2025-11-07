use crate::config::constants::*;
use crate::crypto::signing;
use crate::dirs::{config, contacts, keys};
use crate::server::RelayClient;
use crate::utils::error::{Error, Result};
use crate::utils::file_io;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Serve (send) a file to a trusted contact
pub async fn run(file: PathBuf, to: String, _quiet: bool, local: bool) -> Result<()> {
    println!("{}", " Serving...\n".bright_cyan().bold());

    // Validate file exists
    if !file.exists() {
        return Err(Error::FileNotFound(format!(
            "File not found: {}",
            file.display()
        )));
    }

    if !file.is_file() {
        return Err(Error::InvalidInput(
            "Only single files are supported currently".into(),
        ));
    }

    file_io::validate_file_path(&file).await?;

    let filename = file
        .file_name()
        .ok_or_else(|| Error::InvalidInput("Invalid filename".into()))?
        .to_string_lossy()
        .to_string();

    let metadata = std::fs::metadata(&file)?;
    let filesize = metadata.len();

    // Load config and keys
    let config = config::load_config()?;
    let (signing_key, verifying_key) = keys::load_keys_from(&config.path.keys_path)?;
    let my_fingerprint = hex::encode(verifying_key.to_bytes());

    // Load contacts and find recipient
    let contact_list = contacts::load_contacts()?;
    let recipient = contact_list
        .get(&to)
        .ok_or_else(|| Error::InvalidInput(format!("Contact '{}' not found", to)))?;

    // Display transfer info
    println!("   File: {}", filename.bright_yellow());
    println!(
        "   Size: {} bytes ({:.2} MB)",
        filesize,
        filesize as f64 / (1024.0 * 1024.0)
    );
    println!("   To:   {}", to.bright_white().bold());
    println!(
        "   Key:  {}",
        &recipient.public_key[..16].bright_cyan().dimmed()
    );
    println!();

    // Compute file hash for integrity verification
    println!("{}", " Computing file hash...".bright_cyan());
    let file_hash_hex = file_io::compute_file_hash(&file).await?;
    println!(
        "{}  Hash: {}...",
        "✓".bright_green(),
        &file_hash_hex[..KEY_FINGERPRINT_DISPLAY_LEN]
            .bright_cyan()
            .dimmed()
    );

    println!(
        "{} File hash: {}...",
        "✓".bright_green(),
        &file_hash_hex[..16].bright_cyan().dimmed()
    );
    println!();

    // Create relay client
    let relay_client = if local {
        RelayClient::new(
            "http://localhost:8080".to_string(),
            "localhost".to_string(),
            10000,
        )
    } else {
        RelayClient::new(
            config.server.http_url.clone(),
            config.server.socket_host.clone(),
            config.server.socket_port,
        )
    };

    // Create transfer metadata and signature (includes file hash)
    let metadata_msg = format!("{}|{}|{}", filename, filesize, file_hash_hex);
    let metadata_signature = signing::sign_data(&signing_key, &metadata_msg)?;
    let signature_hex = hex::encode(metadata_signature.to_bytes());

    // Initiate transfer session (blocks until receiver connects)
    // Metadata is sent via HTTP API
    println!("{}", " Waiting for receiver to connect...".yellow());
    let mut session = relay_client
        .serve(
            my_fingerprint.clone(),
            recipient.public_key.clone(),
            filename.clone(),
            filesize,
            signature_hex,
            file_hash_hex,
        )
        .await?;

    println!(
        "{} Receiver connected! Session: {}",
        "✓".bright_green(),
        session.session_id().bright_cyan()
    );
    println!();

    // Socket now ready for binary file transfer
    println!("{} Sending file...", "◆".bright_green());

    // Send file data with progress bar
    let mut file_reader = File::open(&file).await?;
    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(PROGRESS_BAR_TEMPLATE)
            .unwrap()
            .progress_chars(PROGRESS_BAR_CHARS),
    );

    let mut buffer = vec![0u8; FILE_CHUNK_SIZE];
    let mut total_sent = 0u64;

    loop {
        let n = file_reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        session.write_all(&buffer[..n]).await?;
        total_sent += n as u64;
        pb.set_position(total_sent);
    }

    session.flush().await?;
    pb.finish_with_message("Transfer complete!");

    println!();
    println!(" Waiting for receiver confirmation...");

    // Wait for receiver's completion confirmation
    let mut ack_buffer = vec![0u8; 10];
    match session.read(&mut ack_buffer).await {
        Ok(n) if n > 0 && &ack_buffer[..n] == DONE_SIGNAL => {
            println!("{} Receiver confirmed receipt!", "✓".bright_green().bold());
        }
        Ok(n) => {
            println!(
                "{} Got {} bytes, expected DONE signal",
                "✗".bright_yellow().bold(),
                n
            );
            if n > 0 {
                println!(
                    "   Received: {:?}",
                    String::from_utf8_lossy(&ack_buffer[..n])
                );
            }
        }
        Err(e) => {
            println!(
                "{} Failed to read confirmation: {}",
                "✗".bright_red().bold(),
                e
            );
        }
    }

    println!();
    println!(
        "{} File reached successfully! :)",
        "✓".bright_green().bold()
    );
    println!(
        "   Transferred: {} bytes ({:.2} MB)",
        total_sent,
        total_sent as f64 / (1024.0 * 1024.0)
    );

    Ok(())
}
