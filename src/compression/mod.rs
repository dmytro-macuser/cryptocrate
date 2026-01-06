//! Compression utilities using zstd

use crate::error::{CrateError, Result};

/// Default compression level (1-21, higher = better compression but slower)
const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

/// Compress data using zstd
///
/// # Arguments
/// * `data` - Data to compress
/// * `level` - Compression level (1-21, default: 3)
///
/// # Returns
/// Compressed data
pub fn compress(data: &[u8], level: Option<i32>) -> Result<Vec<u8>> {
    let level = level.unwrap_or(DEFAULT_COMPRESSION_LEVEL);
    
    zstd::bulk::compress(data, level)
        .map_err(|e| CrateError::Encryption(format!("Compression failed: {}", e)))
}

/// Decompress zstd-compressed data
///
/// # Arguments
/// * `data` - Compressed data
/// * `max_size` - Maximum decompressed size (safety limit)
///
/// # Returns
/// Decompressed data
pub fn decompress(data: &[u8], max_size: usize) -> Result<Vec<u8>> {
    let decompressed = zstd::bulk::decompress(data, max_size)
        .map_err(|e| CrateError::Decryption(format!("Decompression failed: {}", e)))?;
    
    Ok(decompressed)
}

/// Calculate compression ratio
pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
    if original_size == 0 {
        return 0.0;
    }
    (1.0 - (compressed_size as f64 / original_size as f64)) * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = b"Hello, World! This is a test of compression. ".repeat(100);
        
        let compressed = compress(&original, None).unwrap();
        assert!(compressed.len() < original.len());
        
        let decompressed = decompress(&compressed, original.len() * 2).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compression_ratio() {
        let ratio = compression_ratio(1000, 500);
        assert_eq!(ratio, 50.0);
    }
}
