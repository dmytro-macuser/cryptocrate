//! Directory traversal utilities

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;

/// Represents a file entry with metadata
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub is_dir: bool,
    pub size: u64,
}

/// Collect all files from a path (file or directory)
///
/// # Arguments
/// * `path` - Path to file or directory
/// * `base_path` - Base path for calculating relative paths (usually parent of path)
///
/// # Returns
/// Vector of FileEntry objects
pub fn collect_files<P: AsRef<Path>>(path: P, base_path: Option<P>) -> Result<Vec<FileEntry>> {
    let path = path.as_ref();
    let base = base_path
        .map(|p| p.as_ref().to_path_buf())
        .unwrap_or_else(|| {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });

    let mut entries = Vec::new();

    if path.is_file() {
        // Single file
        let metadata = fs::metadata(path)?;
        let relative_path = path
            .strip_prefix(&base)
            .unwrap_or(path)
            .to_path_buf();

        entries.push(FileEntry {
            path: path.to_path_buf(),
            relative_path,
            is_dir: false,
            size: metadata.len(),
        });
    } else if path.is_dir() {
        // Directory - walk recursively
        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() {
                let metadata = fs::metadata(entry_path)?;
                let relative_path = entry_path
                    .strip_prefix(&base)
                    .unwrap_or(entry_path)
                    .to_path_buf();

                entries.push(FileEntry {
                    path: entry_path.to_path_buf(),
                    relative_path,
                    is_dir: false,
                    size: metadata.len(),
                });
            }
        }
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_collect_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, b"test content").unwrap();

        let files = collect_files(&file_path, None).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].size, 12);
        assert!(!files[0].is_dir);
    }

    #[test]
    fn test_collect_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).unwrap();
        fs::write(dir_path.join("file1.txt"), b"content1").unwrap();
        fs::write(dir_path.join("file2.txt"), b"content2").unwrap();

        let files = collect_files(&dir_path, None).unwrap();
        assert_eq!(files.len(), 2);
    }
}
