//use crate::config::constants::HASH_CHUNK_SIZE;
use crate::utils::error::Result;
use memmap2::Mmap;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::path::PathBuf;
//use tokio::fs::File;
//use tokio::io::AsyncReadExt;

/// Compute SHA256 hash of a file
///
/// Reads the file using memory-mapped I/O for efficiency.
/// Returns hex-encoded hash string.
///
/// # Arguments
/// * `file_path` - Path to the file to hash
///
/// # Returns
/// * `Result<String>` - Hex-encoded SHA256 hash
pub async fn compute_file_hash(file_path: &PathBuf) -> Result<String> {
    let mut hasher = Sha256::new();
    let file = File::open(file_path)?;
    let mmap = unsafe { Mmap::map(&file)? };

    //let mut file = File::open(file_path).await?;
    //let mut buffer = vec![0u8; HASH_CHUNK_SIZE];
    //loop {
    //    let n = file.read(&mut buffer).await?;
    //    if n == 0 {
    //        break;
    //    }
    //    hasher.update(&buffer[..n]);
    //}

    hasher.update(&mmap[..]);

    let hash_result = hasher.finalize();
    Ok(hex::encode(hash_result))
}

/// Validate file path exists and is accessible
///
/// # Arguments
/// * `file_path` - Path to validate
///
/// # Returns
/// * `Result<()>` - Ok if file exists and is readable
pub async fn validate_file_path(file_path: &PathBuf) -> Result<()> {
    if !file_path.exists() {
        return Err(crate::utils::error::Error::FileNotFound(
            file_path.display().to_string(),
        ));
    }

    if !file_path.is_file() {
        return Err(crate::utils::error::Error::InvalidInput(format!(
            "{} is not a file",
            file_path.display()
        )));
    }

    Ok(())
}
