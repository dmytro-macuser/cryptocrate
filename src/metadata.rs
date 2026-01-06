//! File metadata preservation

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use crate::error::Result;

/// File metadata to preserve
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub filename: String,
    pub original_size: u64,
    pub modified_time: Option<u64>,
    pub is_compressed: bool,
}

impl FileMetadata {
    /// Create metadata from a file path
    pub fn from_file<P: AsRef<Path>>(path: P, is_compressed: bool) -> Result<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let modified_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        
        Ok(Self {
            filename,
            original_size: metadata.len(),
            modified_time,
            is_compressed,
        })
    }
    
    /// Serialize metadata to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Filename length (2 bytes)
        let filename_bytes = self.filename.as_bytes();
        bytes.extend_from_slice(&(filename_bytes.len() as u16).to_le_bytes());
        
        // Filename
        bytes.extend_from_slice(filename_bytes);
        
        // Original size (8 bytes)
        bytes.extend_from_slice(&self.original_size.to_le_bytes());
        
        // Modified time (8 bytes, 0 if None)
        bytes.extend_from_slice(&self.modified_time.unwrap_or(0).to_le_bytes());
        
        // Compression flag (1 byte)
        bytes.push(if self.is_compressed { 1 } else { 0 });
        
        bytes
    }
    
    /// Deserialize metadata from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 19 {
            return Err(crate::error::CrateError::InvalidFormat(
                "Metadata too short".to_string(),
            ));
        }
        
        let mut offset = 0;
        
        // Filename length
        let filename_len = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;
        
        // Filename
        if offset + filename_len > bytes.len() {
            return Err(crate::error::CrateError::InvalidFormat(
                "Invalid filename length".to_string(),
            ));
        }
        let filename = String::from_utf8_lossy(&bytes[offset..offset + filename_len]).to_string();
        offset += filename_len;
        
        // Original size
        let original_size = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;
        
        // Modified time
        let time_val = u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let modified_time = if time_val == 0 { None } else { Some(time_val) };
        offset += 8;
        
        // Compression flag
        let is_compressed = bytes[offset] != 0;
        
        Ok(Self {
            filename,
            original_size,
            modified_time,
            is_compressed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_serialization() {
        let metadata = FileMetadata {
            filename: "test.txt".to_string(),
            original_size: 12345,
            modified_time: Some(1234567890),
            is_compressed: true,
        };
        
        let bytes = metadata.to_bytes();
        let deserialized = FileMetadata::from_bytes(&bytes).unwrap();
        
        assert_eq!(deserialized.filename, metadata.filename);
        assert_eq!(deserialized.original_size, metadata.original_size);
        assert_eq!(deserialized.modified_time, metadata.modified_time);
        assert_eq!(deserialized.is_compressed, metadata.is_compressed);
    }
}
