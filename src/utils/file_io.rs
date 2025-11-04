use crate::utils::error::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn compute_hash(file_path: &PathBuf) -> Result<String> {
    let mut hasher = Sha256::new();
    let mut file = File::open(file_path).await?;
    let mut buffer = vec![0u8; 64 * 1024];

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let hash_result = hasher.finalize();
    Ok(hex::encode(hash_result))
}
