// Application Constants

/// Size of chunks when reading/writing files during transfer (1MB)
pub const FILE_CHUNK_SIZE: usize = 1 * 1024 * 1024;

/// Size of chunks when computing file hashes (4MB)
pub const HASH_CHUNK_SIZE: usize = 4 * 1024 * 1024;

/// Buffer size for network transfers (4MB)
pub const BUFFER_SIZE: usize = 4 * 1024 * 1024;

/// Minimum file size allowed for transfer (10MB)
/// Note: This is a soft limit for validation, not enforced by protocol
pub const MIN_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Maximum file size allowed for transfer (10GB)
/// Note: This is a soft limit for validation, not enforced by protocol
pub const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024 * 1024;

// Network Constants

/// Default public server ip address
pub const DEFAULT_PUBLIC_IP: &str = "217.160.136.206";

/// Default relay private server ip address
pub const DEFAULT_PRIVATE_IP: &str = "127.0.0.1";

/// Default relay server http port
pub const DEFAULT_HTTP_PORT: u16 = 8080;

/// Default relay server socket port
pub const DEFAULT_SOCKET_PORT: u16 = 10000;

/// Default Spinner animation
pub const DEFAULT_SPINNER_STYLE: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

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
    "{spinner:.green} |{bar:40.magenta/purple}| ([{percent}%] / [{bytes_per_sec}] / [{elapsed}])";

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
pub const MAX_DONE_WAIT_MILLIS: u64 = 100;

// Validation Constants

/// Maximum length for contact name
pub const MAX_CONTACT_NAME_LEN: usize = 50;

/// Maximum length for filename (for validation)
pub const MAX_FILENAME_LEN: usize = 255;
