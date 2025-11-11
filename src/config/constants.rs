/// Application-wide constants

// File Transfer Constants

/// Size of chunks when reading/writing files during transfer (64KB)
pub const FILE_CHUNK_SIZE: usize = 64 * 1024;

/// Size of chunks when computing file hashes (64KB)
pub const HASH_CHUNK_SIZE: usize = 64 * 1024;

/// Maximum file size allowed for transfer (10GB)
/// Note: This is a soft limit for validation, not enforced by protocol
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024 * 1024;

// Network Constants

/// Default relay server HTTP URL
pub const DEFAULT_HTTP_URL: &str = "http://217.160.136.206:8080";

/// Default relay server socket host
pub const DEFAULT_SOCKET_HOST: &str = "217.160.136.206";

pub const DEFAULT_SPINNER_STYLE: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

/// Default relay server socket port
pub const DEFAULT_SOCKET_PORT: u16 = 10000;

// Cryptographic Constants

/// Length of Ed25519 public key in bytes
pub const ED25519_PUBLIC_KEY_LEN: usize = 32;

/// Length of Ed25519 private key in bytes
pub const ED25519_PRIVATE_KEY_LEN: usize = 64;

/// Length of Ed25519 signature in bytes
pub const ED25519_SIGNATURE_LEN: usize = 64;

/// Length of SHA256 hash in bytes
pub const SHA256_HASH_LEN: usize = 32;

/// Number of hex characters to display for key fingerprints
pub const KEY_FINGERPRINT_DISPLAY_LEN: usize = 16;

// UI/Display Constants

/// Progress bar template
pub const PROGRESS_BAR_TEMPLATE: &str =
    "{spinner:.green} |{bar:40.magenta/purple}| ([{percent}%] / [{bytes_per_sec}] / [{eta}])";

/// Progress bar characters
pub const PROGRESS_BAR_CHARS: &str = "░▒▓█";

/// Application name for display
pub const APP_NAME: &str = "R-Share";

/// Application version
pub const APP_VERSION: &str = "1.0.0-beta";

// Session & Protocol Constants

/// Protocol delimiter for socket messages
pub const PROTOCOL_DELIMITER: &str = "\n";

/// DONE signal sent by receiver after successful transfer
pub const DONE_SIGNAL: &[u8] = b"DONE\n";

/// READY signal sent by receiver when ready to receive
pub const READY_SIGNAL: &[u8] = b"READY\n";

/// ACK signal sent by sender acknowledging READY
pub const ACK_SIGNAL: &[u8] = b"ACK\n";

/// Error signal prefix
pub const ERROR_SIGNAL_PREFIX: &str = "ERROR:";

/// Maximum time to wait for DONE signal from receiver (seconds)
pub const MAX_DONE_WAIT_SECS: u64 = 30;

// Validation Constants

/// Maximum length for contact name
pub const MAX_CONTACT_NAME_LEN: usize = 50;

/// Maximum length for filename (for validation)
pub const MAX_FILENAME_LEN: usize = 255;

/// Minimum file size (1 byte)
pub const MIN_FILE_SIZE: u64 = 1;
