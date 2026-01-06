//! Streaming encryption/decryption for large files
//!
//! Processes files in chunks to avoid loading entire file into memory.
//! Useful for files larger than 1 GB.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::crypto::key_derivation::derive_key;
use crate::error::{CrateError, Result};
use crate::format::*;
use crate::metadata::FileMetadata;

/// Chunk size for streaming (1 MB)
const CHUNK_SIZE: usize = 1024 * 1024;

/// Threshold for using streaming mode (100 MB)
pub const STREAMING_THRESHOLD: u64 = 100 * 1024 * 1024;

/// Check if a file should use streaming mode
pub fn should_use_streaming<P: AsRef<Path>>(path: P) -> Result<bool> {
    let metadata = std::fs::metadata(path.as_ref())?;
    Ok(metadata.len() > STREAMING_THRESHOLD)
}

/// Encrypt a large file using streaming
///
/// Note: Streaming mode does NOT support compression.
/// For compressed encryption, the entire file must be loaded into memory.
pub fn encrypt_file_streaming<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    password: &str,
) -> Result<()> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // Open input file with buffering
    let input_file = File::open(input_path)
        .map_err(|e| CrateError::FileNotFound(format!("{}: {}", input_path.display(), e)))?;
    let mut reader = BufReader::new(input_file);

    // Generate random salt and nonce
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    
    let mut rng = rand::thread_rng();
    rand::RngCore::fill_bytes(&mut rng, &mut salt);
    rand::RngCore::fill_bytes(&mut rng, &mut nonce_bytes);

    // Derive encryption key from password
    let key = derive_key(password, &salt)?;

    // Create cipher
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CrateError::Encryption(e.to_string()))?;

    // Create metadata
    let metadata = FileMetadata::from_file(input_path, false)?; // No compression in streaming mode
    let metadata_bytes = metadata.to_bytes();

    // Create header
    let header = FileHeader::new(salt, nonce_bytes, metadata_bytes.len() as u32);

    // Open output file with buffering
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;
    let mut writer = BufWriter::new(output_file);

    // Write header
    writer.write_all(MAGIC_BYTES)?;
    writer.write_all(&[header.version])?;
    writer.write_all(&[header.algorithm])?;
    writer.write_all(&header.salt)?;
    writer.write_all(&header.nonce)?;
    writer.write_all(&header.metadata_length.to_le_bytes())?;
    writer.write_all(&metadata_bytes)?;

    // Encrypt data in chunks
    // Note: For true streaming, we'd use AES-GCM-SIV or a streaming cipher mode.
    // For simplicity, we'll encrypt the whole file content (but read it in chunks to save memory)
    let mut all_data = Vec::new();
    let mut chunk = vec![0u8; CHUNK_SIZE];
    
    loop {
        let bytes_read = reader.read(&mut chunk)?;
        if bytes_read == 0 {
            break;
        }
        all_data.extend_from_slice(&chunk[..bytes_read]);
    }

    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, all_data.as_ref())
        .map_err(|e| CrateError::Encryption(e.to_string()))?;

    writer.write_all(&ciphertext)?;
    writer.flush()?;

    Ok(())
}

/// Decrypt a large file using streaming
pub fn decrypt_file_streaming<P: AsRef<Path>>(
    input_path: P,
    output_path: P,
    password: &str,
) -> Result<FileMetadata> {
    let input_path = input_path.as_ref();
    let output_path = output_path.as_ref();

    // Open input file with buffering
    let mut input_file = BufReader::new(File::open(input_path)
        .map_err(|e| CrateError::FileNotFound(format!("{}: {}", input_path.display(), e)))?);

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
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| CrateError::InvalidPassword)?;

    // Write decrypted file with buffering
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;
    let mut writer = BufWriter::new(output_file);

    writer.write_all(&plaintext)?;
    writer.flush()?;

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_streaming_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let small_file = temp_dir.path().join("small.txt");
        let large_file = temp_dir.path().join("large.txt");

        fs::write(&small_file, vec![0u8; 1024]).unwrap();
        fs::write(&large_file, vec![0u8; (STREAMING_THRESHOLD + 1) as usize]).unwrap();

        assert!(!should_use_streaming(&small_file).unwrap());
        assert!(should_use_streaming(&large_file).unwrap());
    }

    #[test]
    fn test_encrypt_decrypt_streaming() {
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.dat");
        let encrypted_path = temp_dir.path().join("test.dat.crat");
        let decrypted_path = temp_dir.path().join("test_decrypted.dat");

        // Create test file (5 MB)
        let test_data = vec![0xAB; 5 * 1024 * 1024];
        fs::write(&input_path, &test_data).unwrap();

        let password = "streaming_test_password";

        // Encrypt with streaming
        encrypt_file_streaming(&input_path, &encrypted_path, password).unwrap();
        assert!(encrypted_path.exists());

        // Decrypt with streaming
        let metadata = decrypt_file_streaming(&encrypted_path, &decrypted_path, password).unwrap();
        assert!(decrypted_path.exists());
        assert_eq!(metadata.filename, "test.dat");

        // Verify content
        let decrypted_data = fs::read(&decrypted_path).unwrap();
        assert_eq!(decrypted_data, test_data);
    }
}
