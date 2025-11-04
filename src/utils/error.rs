use std::fmt;

/// Custom error type for rshare
#[derive(Debug)]
pub enum Error {
    /// File system errors
    FileNotFound(String),
    FileRead(String),
    FileWrite(String),
    FileDelete(String),

    /// Network errors
    NetworkError(String),
    ConnectionFailed(String),
    SocketError(String),
    HttpError(String),
    Timeout(String),

    /// Crypto errors
    KeyGenerationFailed(String),
    SignatureError(String),
    HashMismatch {
        expected: String,
        got: String,
    },
    InvalidSignature(String),

    /// Input validation errors
    InvalidInput(String),
    InvalidKey(String),
    InvalidConfig(String),

    /// Configuration errors
    ConfigError(String),
    CryptoError(String),

    /// Session/Transfer errors
    SessionNotFound(String),
    TransferInterrupted(String),
    PartnerNotFound(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            Error::FileRead(msg) => write!(f, "Failed to read file: {}", msg),
            Error::FileWrite(msg) => write!(f, "Failed to write file: {}", msg),
            Error::FileDelete(msg) => write!(f, "Failed to delete file: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Error::SocketError(msg) => write!(f, "Socket error: {}", msg),
            Error::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            Error::Timeout(msg) => write!(f, "Operation timed out: {}", msg),
            Error::KeyGenerationFailed(msg) => write!(f, "Key generation failed: {}", msg),
            Error::SignatureError(msg) => write!(f, "Signature error: {}", msg),
            Error::HashMismatch { expected, got } => {
                write!(
                    f,
                    "Hash mismatch - Expected: {}..., Got: {}...",
                    &expected[..16.min(expected.len())],
                    &got[..16.min(got.len())]
                )
            }
            Error::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Error::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            Error::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Error::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            Error::SessionNotFound(msg) => write!(f, "Session not found: {}", msg),
            Error::TransferInterrupted(msg) => write!(f, "Transfer interrupted: {}", msg),
            Error::PartnerNotFound(msg) => write!(f, "Transfer partner not found: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Convert std::io::Error to our Error type
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Error::FileRead(err.to_string()),
            _ => Error::NetworkError(err.to_string()),
        }
    }
}

/// Convert toml errors
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::ConfigError(err.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::ConfigError(err.to_string())
    }
}

/// Convert serde_json errors
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::ConfigError(err.to_string())
    }
}

/// Convert reqwest errors
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Error::Timeout(err.to_string())
        } else if err.is_connect() {
            Error::ConnectionFailed(err.to_string())
        } else {
            Error::HttpError(err.to_string())
        }
    }
}

/// Custom Result type
pub type Result<T> = std::result::Result<T, Error>;
