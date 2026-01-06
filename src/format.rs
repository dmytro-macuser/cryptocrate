//! File format constants and structures

/// Magic bytes identifying a CryptoCrate file: "CRAT"
pub const MAGIC_BYTES: &[u8; 4] = b"CRAT";

/// Current file format version
pub const VERSION: u8 = 1;

/// Algorithm identifier for AES-256-GCM
pub const ALGORITHM_AES256_GCM: u8 = 1;

/// Salt length for Argon2 (32 bytes)
pub const SALT_LENGTH: usize = 32;

/// Nonce length for AES-GCM (12 bytes)
pub const NONCE_LENGTH: usize = 12;

/// AES-256 key length (32 bytes)
pub const KEY_LENGTH: usize = 32;

/// GCM authentication tag length (16 bytes)
pub const TAG_LENGTH: usize = 16;

/// Total header size (without metadata)
pub const HEADER_SIZE: usize = 4 + 1 + 1 + SALT_LENGTH + NONCE_LENGTH + 4; // 54 bytes

/// File header structure
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub version: u8,
    pub algorithm: u8,
    pub salt: [u8; SALT_LENGTH],
    pub nonce: [u8; NONCE_LENGTH],
    pub metadata_length: u32,
}

impl FileHeader {
    pub fn new(salt: [u8; SALT_LENGTH], nonce: [u8; NONCE_LENGTH], metadata_length: u32) -> Self {
        Self {
            version: VERSION,
            algorithm: ALGORITHM_AES256_GCM,
            salt,
            nonce,
            metadata_length,
        }
    }
}
