//! Secure file deletion with multiple overwrite passes
//!
//! Implements secure deletion by overwriting file contents multiple times
//! before deletion to prevent data recovery.

use rand::RngCore;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

use crate::error::Result;

/// Number of overwrite passes for secure deletion
const DEFAULT_OVERWRITE_PASSES: u32 = 3;

/// Secure deletion modes
#[derive(Debug, Clone, Copy)]
pub enum SecureDeleteMode {
    /// Quick: 1 pass with random data
    Quick,
    /// Standard: 3 passes (random, zeros, random)
    Standard,
    /// Paranoid: 7 passes (DoD 5220.22-M standard)
    Paranoid,
}

impl SecureDeleteMode {
    pub fn passes(&self) -> u32 {
        match self {
            SecureDeleteMode::Quick => 1,
            SecureDeleteMode::Standard => 3,
            SecureDeleteMode::Paranoid => 7,
        }
    }
}

/// Securely delete a file by overwriting it multiple times
pub fn secure_delete<P: AsRef<Path>>(path: P, mode: SecureDeleteMode) -> Result<()> {
    let path = path.as_ref();
    
    // Get file size
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len() as usize;
    
    if file_size == 0 {
        // Empty file, just delete it
        std::fs::remove_file(path)?;
        return Ok(());
    }
    
    // Open file for writing
    let mut file = OpenOptions::new()
        .write(true)
        .open(path)?;
    
    let passes = mode.passes();
    let mut rng = rand::thread_rng();
    
    // Perform overwrite passes
    for pass in 0..passes {
        file.seek(SeekFrom::Start(0))?;
        
        // Determine what to write for this pass
        let pattern = match mode {
            SecureDeleteMode::Quick => OverwritePattern::Random,
            SecureDeleteMode::Standard => {
                match pass {
                    0 => OverwritePattern::Random,
                    1 => OverwritePattern::Zeros,
                    _ => OverwritePattern::Random,
                }
            }
            SecureDeleteMode::Paranoid => {
                match pass {
                    0 => OverwritePattern::Random,
                    1 => OverwritePattern::Ones,
                    2 => OverwritePattern::Random,
                    3 => OverwritePattern::Pattern(0xAA),
                    4 => OverwritePattern::Pattern(0x55),
                    5 => OverwritePattern::Random,
                    _ => OverwritePattern::Random,
                }
            }
        };
        
        overwrite_with_pattern(&mut file, file_size, pattern, &mut rng)?;
        file.sync_all()?;
    }
    
    // Close and delete the file
    drop(file);
    std::fs::remove_file(path)?;
    
    Ok(())
}

/// Overwrite patterns
enum OverwritePattern {
    Random,
    Zeros,
    Ones,
    Pattern(u8),
}

/// Overwrite file with specified pattern
fn overwrite_with_pattern<R: RngCore>(
    file: &mut File,
    size: usize,
    pattern: OverwritePattern,
    rng: &mut R,
) -> Result<()> {
    const BUFFER_SIZE: usize = 64 * 1024; // 64 KB buffer
    let mut buffer = vec![0u8; BUFFER_SIZE];
    
    let mut remaining = size;
    
    while remaining > 0 {
        let write_size = remaining.min(BUFFER_SIZE);
        let buf = &mut buffer[..write_size];
        
        // Fill buffer with pattern
        match pattern {
            OverwritePattern::Random => rng.fill_bytes(buf),
            OverwritePattern::Zeros => buf.fill(0x00),
            OverwritePattern::Ones => buf.fill(0xFF),
            OverwritePattern::Pattern(byte) => buf.fill(byte),
        }
        
        file.write_all(buf)?;
        remaining -= write_size;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_secure_delete_quick() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        fs::write(&file_path, b"sensitive data").unwrap();
        assert!(file_path.exists());
        
        // Securely delete it
        secure_delete(&file_path, SecureDeleteMode::Quick).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_secure_delete_standard() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a larger test file
        let data = vec![0xAB; 1024 * 100]; // 100 KB
        fs::write(&file_path, &data).unwrap();
        
        // Securely delete it
        secure_delete(&file_path, SecureDeleteMode::Standard).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_secure_delete_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        
        // Create an empty file
        File::create(&file_path).unwrap();
        
        // Should delete without error
        secure_delete(&file_path, SecureDeleteMode::Quick).unwrap();
        assert!(!file_path.exists());
    }
}
