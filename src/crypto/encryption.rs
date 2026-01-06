//! File encryption and decryption using AES-256-GCM

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use crate::compression;
use crate::crypto::key_derivation::derive_key;
use crate::error::{CrateError, Result};
use crate::format::*;
use crate::metadata::FileMetadata;

/// Maximum decompressed file size (1 GB safety limit)
const MAX_DECOMPRESSED_SIZE: usize = 1024 * 1024 * 1024;

/// Encrypt a file with AES-256-GCM
///
/// # Arguments
/// * `input_path` - Path to the file to encrypt
/// * `output_path` - Path where the encrypted file will be saved
/// * `password` - Password for encryption
/// * `compress` - Whether to compress before encrypting
///
/// # Returns
/// Ok(()) on success, or an error
pub fn encrypt_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    password: &str,
    compress: bool,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // Read input file
    let mut input_file = File::open(input_path)
        .map_err(|e| CrateError::FileNotFound(format!("{}: {}", input_path.display(), e)))?;

    let mut plaintext = Vec::new();
    input_file.read_to_end(&mut plaintext)?;

    // Compress if requested
    let data_to_encrypt = if compress && !plaintext.is_empty() {
        compression::compress(&plaintext, None)?
    } else {
        plaintext.clone()
    };

    // Generate random salt and nonce
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce_bytes = [0u8; NONCE_LENGTH];

    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut salt);
    rng.fill_bytes(&mut nonce_bytes);

    // Derive encryption key from password
    let key = derive_key(password, &salt)?;

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CrateError::Encryption(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the data
    let ciphertext = cipher
        .encrypt(nonce, data_to_encrypt.as_ref())
        .map_err(|e| CrateError::Encryption(e.to_string()))?;

    // Create metadata
    let metadata = FileMetadata::from_file(input_path, compress)?;
    let metadata_bytes = metadata.to_bytes();

    // Create header
    let header = FileHeader::new(salt, nonce_bytes, metadata_bytes.len() as u32);

    // Write encrypted file
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;

    // Write magic bytes
    output_file.write_all(MAGIC_BYTES)?;

    // Write version
    output_file.write_all(&[header.version])?;

    // Write algorithm
    output_file.write_all(&[header.algorithm])?;

    // Write salt
    output_file.write_all(&header.salt)?;

    // Write nonce
    output_file.write_all(&header.nonce)?;

    // Write metadata length
    output_file.write_all(&header.metadata_length.to_le_bytes())?;

    // Write metadata
    output_file.write_all(&metadata_bytes)?;

    // Write encrypted data
    output_file.write_all(&ciphertext)?;

    output_file.flush()?;

    Ok(())
}

/// Decrypt a file encrypted with CryptoCrate
///
/// # Arguments
/// * `input_path` - Path to the encrypted .crat file
/// * `output_path` - Path where the decrypted file will be saved
/// * `password` - Password for decryption
///
/// # Returns
/// Ok(FileMetadata) on success, or an error
pub fn decrypt_file<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    password: &str,
) -> Result<FileMetadata> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // Read encrypted file
    let mut input_file = File::open(input_path)
        .map_err(|e| CrateError::FileNotFound(format!("{}: {}", input_path.display(), e)))?;

    // Read and verify magic bytes
    let mut magic = [0u8; 4];
    input_file.read_exact(&mut magic)?;
    if &magic != MAGIC_BYTES {
        return Err(CrateError::InvalidFormat(
            "Not a valid CryptoCrate file".to_string(),
        ));
    }

    // Read version
    let mut version = [0u8; 1];
    input_file.read_exact(&mut version)?;
    if version[0] != VERSION {
        return Err(CrateError::UnsupportedVersion(version[0]));
    }

    // Read algorithm
    let mut algorithm = [0u8; 1];
    input_file.read_exact(&mut algorithm)?;
    if algorithm[0] != ALGORITHM_AES256_GCM {
        return Err(CrateError::InvalidFormat(
            "Unsupported encryption algorithm".to_string(),
        ));
    }

    // Read salt
    let mut salt = [0u8; SALT_LENGTH];
    input_file.read_exact(&mut salt)?;

    // Read nonce
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    input_file.read_exact(&mut nonce_bytes)?;

    // Read metadata length
    let mut metadata_len_bytes = [0u8; 4];
    input_file.read_exact(&mut metadata_len_bytes)?;
    let metadata_len = u32::from_le_bytes(metadata_len_bytes) as usize;

    // Read metadata
    let mut metadata_bytes = vec![0u8; metadata_len];
    input_file.read_exact(&mut metadata_bytes)?;
    let metadata = FileMetadata::from_bytes(&metadata_bytes)?;

    // Read encrypted data
    let mut ciphertext = Vec::new();
    input_file.read_to_end(&mut ciphertext)?;

    // Derive decryption key
    let key = derive_key(password, &salt)?;

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CrateError::Decryption(e.to_string()))?;

    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt the data
    let decrypted_data = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| CrateError::InvalidPassword)?;

    // Decompress if needed
    let plaintext = if metadata.is_compressed {
        compression::decompress(&decrypted_data, MAX_DECOMPRESSED_SIZE)?
    } else {
        decrypted_data
    };

    // Write decrypted file
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;

    output_file.write_all(&plaintext)?;
    output_file.flush()?;

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.txt.crat");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");

        // Create test file
        let test_data = b"Hello, CryptoCrate! This is a test.";
        fs::write(&input_path, test_data).unwrap();

        let password = "super_secret_password";

        // Encrypt without compression
        encrypt_file(&input_path, &encrypted_path, password, false).unwrap();
        assert!(encrypted_path.exists());

        // Decrypt
        let metadata = decrypt_file(&encrypted_path, &decrypted_path, password).unwrap();
        assert!(decrypted_path.exists());
        assert_eq!(metadata.filename, "test.txt");
        assert!(!metadata.is_compressed);

        // Verify content
        let decrypted_data = fs::read(&decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_encrypt_decrypt_with_compression() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.txt.crat");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");

        // Create test file with repeating content (compresses well)
        let test_data = b"Hello, World! ".repeat(100);
        fs::write(&input_path, &test_data).unwrap();

        let password = "super_secret_password";

        // Encrypt with compression
        encrypt_file(&input_path, &encrypted_path, password, true).unwrap();

        // Decrypt
        let metadata = decrypt_file(&encrypted_path, &decrypted_path, password).unwrap();
        assert!(metadata.is_compressed);

        // Verify content
        let decrypted_data = fs::read(&decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);
    }

    #[test]
    fn test_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test.txt.crat");
        let decrypted_path = temp_dir.path().join("test_decrypted.txt");

        fs::write(&input_path, b"Secret data").unwrap();

        encrypt_file(&input_path, &encrypted_path, "correct_password", false).unwrap();

        let result = decrypt_file(&encrypted_path, &decrypted_path, "wrong_password");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CrateError::InvalidPassword));
    }
}
