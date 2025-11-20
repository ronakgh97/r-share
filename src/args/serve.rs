use crate::config::constants::*;
use crate::crypto::{encryption, key_exchange, signing};
#[allow(unused_imports)]
//use std::fs::File;
use crate::dirs::config::Config;
use crate::dirs::{config, contacts, keys};
use crate::server::RelayClient;
use crate::utils::error::{Error, Result};
use crate::utils::hash::{self, validate_file_path};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
#[allow(unused_imports)]
use memmap2::Mmap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};

/// Serve (send) a file to a trusted contact
pub async fn run(file: PathBuf, to: String, _quiet: bool, relay: Option<String>) -> Result<()> {
    println!("{}", "Serving...\n".bright_blue().bold());

    // Validate file exists
    validate_file_path(&file).await?;

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
        .ok_or_else(|| Error::InvalidInput(format!("Contact >'{}'< not found", to)))?;

    // Compute file hash for integrity verification
    //println!("{}", " Computing file hash...".bright_cyan());
    let file_hash_hex = hash::compute_file_hash(&file).await?;
    //println!(
    //    "{}  Hash: {}...",
    //    "✓".bright_green(),
    //    &file_hash_hex[..KEY_FINGERPRINT_DISPLAY_LEN]
    //        .bright_cyan()
    //        .dimmed()
    //);
    //println!();

    // Display transfer info
    println!(
        " File: {} | Hash {}",
        filename.bright_yellow(),
        file_hash_hex[..KEY_FINGERPRINT_DISPLAY_LEN]
            .bright_cyan()
            .dimmed()
    );
    println!(
        " Size: {} bytes ({:.2} MB)",
        filesize,
        filesize as f64 / (1024.0 * 1024.0)
    );
    println!(" To:   {}", to.bright_white().bold());
    //println!(
    //    "   Key:  {}...",
    //    &recipient.public_key[..16].bright_cyan().dimmed()
    //);
    //println!();

    // Select relay server from config
    let server_config = Config::select_server(&config, relay)?;

    // Create relay client
    let relay_client = RelayClient::new(
        server_config.server_ip,
        server_config.http_port,
        server_config.socket_port,
    );

    // Create transfer metadata and signature (includes file hash)
    let metadata_msg = format!("{}|{}|{}", filename, filesize, file_hash_hex);
    let metadata_signature = signing::sign_data(&signing_key, &metadata_msg)?;
    let signature_hex = hex::encode(metadata_signature.to_bytes());

    // Generate ephemeral X25519 keypair for this transfer
    //println!(
    //    "{}",
    //    " Generating ephemeral encryption keys...".bright_cyan()
    //);

    let ephemeral_keypair = key_exchange::EphemeralKeyPair::generate();
    let sender_ephemeral_hex = ephemeral_keypair.public_key_hex();

    //println!(
    //    "{}  Ephemeral key: {}...",
    //    "✓".bright_green(),
    //    &sender_ephemeral_hex[..16].bright_cyan().dimmed()
    //);
    //println!();

    // Initiate transfer session (blocks until receiver connects)
    // Metadata is sent via HTTP API
    println!();
    println!("{}", "Waiting for receiver to connect...".yellow());
    let mut session = relay_client
        .serve(
            my_fingerprint.clone(),
            recipient.public_key.clone(),
            filename.clone(),
            filesize,
            signature_hex,
            file_hash_hex,
            sender_ephemeral_hex,
        )
        .await?;

    println!("  Session: {}", session.session_id().bright_blue());
    println!();

    // Derive encryption key from ephemeral keys
    let receiver_ephemeral_hex = session
        .receiver_ephemeral_key
        .as_ref()
        .ok_or_else(|| Error::SessionError(format!("Receiver key not found")))?;

    //println!("{}", " Deriving encryption key...".bright_cyan());
    let aes_key = key_exchange::perform_key_exchange(
        ephemeral_keypair.secret,
        receiver_ephemeral_hex,
        session.session_id(),
    )?;
    //println!("{}  Encryption key derived", "✓".bright_green());
    //println!();

    // Socket now ready for encrypted binary file transfer
    println!(
        "{} Encrypting and sending file...",
        "⇗".bright_magenta().bold()
    );

    // Send file data with progress bar (encrypt each chunk)
    //let file_reader = File::open(&file)?;
    let file_reader = File::open(&file).await?;
    let mut buf_reader = BufReader::with_capacity(BUFFER_SIZE, file_reader);
    //let mmap = unsafe { Mmap::map(&file_reader)? };

    let pb = ProgressBar::new(filesize);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(PROGRESS_BAR_TEMPLATE)
            .unwrap()
            .progress_chars(PROGRESS_BAR_CHARS)
            .tick_chars(DEFAULT_SPINNER_STYLE),
    );

    let mut buffer = vec![0u8; FILE_CHUNK_SIZE];
    let mut total_sent = 0u64;

    loop {
        let n = buf_reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        //for chunk in mmap.chunks(FILE_CHUNK_SIZE) {
        // Encrypt the chunk before sending
        let encrypted_chunk = encryption::encrypt_chunk(&aes_key, &buffer[..n])?;
        //let encrypted_chunk = encryption::encrypt_chunk(&aes_key, chunk)?;

        // Send encrypted chunk size (4 bytes) followed by encrypted data
        let chunk_size = encrypted_chunk.len() as u32;
        session.write_all(&chunk_size.to_be_bytes()).await?;
        session.write_all(&encrypted_chunk).await?;

        total_sent += n as u64;
        //total_sent += chunk.len() as u64;
        pb.set_position(total_sent);
    }

    session.flush().await?;
    pb.finish_with_message("Transfer complete!");

    println!();
    println!();
    println!("{}", "Waiting for receiver confirmation....".yellow());

    // Wait for receiver's completion confirmation
    let mut ack_buffer = vec![0u8; 10];
    match session.read(&mut ack_buffer).await {
        Ok(n) if n > 0 && &ack_buffer[..n] == DONE_SIGNAL => {
            println!("  Receiver confirmed receipt!");
        }
        Ok(n) => {
            println!(
                //"{} Got {} bytes, expected DONE signal",
                "{} Unexpected confirmation response: {} bytes",
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
        Err(_e) => {
            println!("{} Failed to read confirmation", "✗".bright_red().bold(),);
        }
    }

    println!();
    println!("{} File reached successfully", "✓".bright_green().bold());
    //println!(
    //    "   Transferred: {} bytes ({:.2} MB)",
    //    total_sent,
    //    total_sent as f64 / (1024.0 * 1024.0)
    //);

    Ok(())
}
