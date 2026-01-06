//! Key file support for enhanced security
//!
//! Key files can be used alone or combined with passwords for two-factor authentication.
//! A key file is a binary file containing random data used as part of the encryption key.

use rand::RngCore;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::error::{CrateError, Result};

/// Default key file size in bytes (4 KB)
pub const DEFAULT_KEYFILE_SIZE: usize = 4096;

/// Maximum key file size (10 MB)
const MAX_KEYFILE_SIZE: usize = 10 * 1024 * 1024;

/// Generate a new random key file
pub fn generate_keyfile<P: AsRef<Path>>(path: P, size: Option<usize>) -> Result<()> {
    let size = size.unwrap_or(DEFAULT_KEYFILE_SIZE);
    
    if size > MAX_KEYFILE_SIZE {
        return Err(CrateError::InvalidFormat(
            format!("Key file size too large (max {} bytes)", MAX_KEYFILE_SIZE),
        ));
    }
    
    // Generate random data
    let mut key_data = vec![0u8; size];
    let mut rng = rand::thread_rng();
    rng.fill_bytes(&mut key_data);
    
    // Write to file
    let mut file = File::create(path.as_ref())?;
    file.write_all(&key_data)?;
    file.sync_all()?;
    
    Ok(())
}

/// Read and hash a key file to produce a 32-byte key component
pub fn read_keyfile<P: AsRef<Path>>(path: P) -> Result<[u8; 32]> {
    let path = path.as_ref();
    
    // Check file size
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_KEYFILE_SIZE as u64 {
        return Err(CrateError::InvalidFormat(
            format!("Key file too large (max {} bytes)", MAX_KEYFILE_SIZE),
        ));
    }
    
    if metadata.len() == 0 {
        return Err(CrateError::InvalidFormat(
            "Key file is empty".to_string(),
        ));
    }
    
    // Read key file
    let mut file = File::open(path)
        .map_err(|e| CrateError::FileNotFound(format!("Key file: {}", e)))?;
    
    let mut key_data = Vec::new();
    file.read_to_end(&mut key_data)?;
    
    // Hash the key file content to get a consistent 32-byte key
    let mut hasher = Sha256::new();
    hasher.update(&key_data);
    let hash = hasher.finalize();
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&hash);
    Ok(result)
}

/// Combine password and key file into a single key material
/// Uses HKDF-like approach: hash(password || keyfile_hash)
pub fn combine_password_and_keyfile(password: &str, keyfile_hash: &[u8; 32]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(keyfile_hash);
    let combined = hasher.finalize();
    combined.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_and_read_keyfile() {
        let temp_dir = TempDir::new().unwrap();
        let keyfile_path = temp_dir.path().join("test.key");

        // Generate key file
        generate_keyfile(&keyfile_path, Some(1024)).unwrap();
        assert!(keyfile_path.exists());

        // Read key file
        let key1 = read_keyfile(&keyfile_path).unwrap();
        assert_eq!(key1.len(), 32);

        // Reading same file should give same hash
        let key2 = read_keyfile(&keyfile_path).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_combine_password_and_keyfile() {
        let password = "test_password";
        let keyfile_hash = [42u8; 32];

        let combined1 = combine_password_and_keyfile(password, &keyfile_hash);
        let combined2 = combine_password_and_keyfile(password, &keyfile_hash);
        
        assert_eq!(combined1, combined2);
        assert_eq!(combined1.len(), 32);

        // Different password should give different result
        let combined3 = combine_password_and_keyfile("different", &keyfile_hash);
        assert_ne!(combined1, combined3);
    }

    #[test]
    fn test_empty_keyfile() {
        let temp_dir = TempDir::new().unwrap();
        let keyfile_path = temp_dir.path().join("empty.key");
        File::create(&keyfile_path).unwrap();

        let result = read_keyfile(&keyfile_path);
        assert!(result.is_err());
    }
}
