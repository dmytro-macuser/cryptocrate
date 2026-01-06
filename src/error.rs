//! Error types for CryptoCrate

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CrateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Key derivation error: {0}")]
    KeyDerivation(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),
}

pub type Result<T> = std::result::Result<T, CrateError>;
