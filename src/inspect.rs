//! File inspection utilities - view encrypted file metadata without decrypting

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{CrateError, Result};
use crate::format::*;
use crate::metadata::FileMetadata;

/// Information about an encrypted file
#[derive(Debug)]
pub struct FileInfo {
    pub version: u8,
    pub algorithm: String,
    pub metadata: FileMetadata,
    pub encrypted_size: u64,
}

impl FileInfo {
    /// Display the file information in a human-readable format
    pub fn display(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("📦 File Format: CryptoCrate v{}\n", self.version));
        output.push_str(&format!("🔐 Algorithm: {}\n", self.algorithm));
        output.push_str(&format!("📄 Original Filename: {}\n", self.metadata.filename));
        output.push_str(&format!("📏 Original Size: {}\n", format_size(self.metadata.original_size)));
        output.push_str(&format!("📦 Encrypted Size: {}\n", format_size(self.encrypted_size)));
        
        if let Some(modified) = self.metadata.modified_time {
            let datetime = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(modified);
            if let Ok(duration) = datetime.duration_since(UNIX_EPOCH) {
                output.push_str(&format!("📅 Modified: {} (Unix: {})\n", 
                    format_timestamp(duration.as_secs()), modified));
            }
        }
        
        output.push_str(&format!("🗜️  Compressed: {}\n", 
            if self.metadata.is_compressed { "Yes" } else { "No" }));
        
        if self.metadata.is_compressed {
            let ratio = ((self.encrypted_size as f64 / self.metadata.original_size as f64) * 100.0);
            output.push_str(&format!("📊 Compression Ratio: {:.1}% of original\n", ratio));
        }
        
        output
    }
}

/// Inspect an encrypted file without decrypting it
pub fn inspect_file<P: AsRef<Path>>(path: P) -> Result<FileInfo> {
    let path = path.as_ref();
    
    // Get file size
    let encrypted_size = std::fs::metadata(path)?.len();
    
    // Read encrypted file
    let mut input_file = File::open(path)
        .map_err(|e| CrateError::FileNotFound(format!("{}: {}", path.display(), e)))?;

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
    let algorithm_name = match algorithm[0] {
        ALGORITHM_AES256_GCM => "AES-256-GCM",
        _ => "Unknown",
    };

    // Skip salt and nonce (we don't need them for inspection)
    let mut skip_buffer = vec![0u8; SALT_LENGTH + NONCE_LENGTH];
    input_file.read_exact(&mut skip_buffer)?;

    // Read metadata length
    let mut metadata_len_bytes = [0u8; 4];
    input_file.read_exact(&mut metadata_len_bytes)?;
    let metadata_len = u32::from_le_bytes(metadata_len_bytes) as usize;

    // Read metadata
    let mut metadata_bytes = vec![0u8; metadata_len];
    input_file.read_exact(&mut metadata_bytes)?;
    let metadata = FileMetadata::from_bytes(&metadata_bytes)?;

    Ok(FileInfo {
        version: version[0],
        algorithm: algorithm_name.to_string(),
        metadata,
        encrypted_size,
    })
}

/// Format file size for display
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format Unix timestamp to human-readable date
fn format_timestamp(timestamp: u64) -> String {
    use std::time::Duration;
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    
    let diff = if now > timestamp {
        now - timestamp
    } else {
        timestamp - now
    };
    
    const MINUTE: u64 = 60;
    const HOUR: u64 = MINUTE * 60;
    const DAY: u64 = HOUR * 24;
    const WEEK: u64 = DAY * 7;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = DAY * 365;
    
    let relative = if diff < MINUTE {
        "just now".to_string()
    } else if diff < HOUR {
        format!("{} minutes ago", diff / MINUTE)
    } else if diff < DAY {
        format!("{} hours ago", diff / HOUR)
    } else if diff < WEEK {
        format!("{} days ago", diff / DAY)
    } else if diff < MONTH {
        format!("{} weeks ago", diff / WEEK)
    } else if diff < YEAR {
        format!("{} months ago", diff / MONTH)
    } else {
        format!("{} years ago", diff / YEAR)
    };
    
    relative
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }
}
