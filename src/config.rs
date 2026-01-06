//! Configuration management

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CrateError, Result};

/// Default configuration file name
const CONFIG_FILE_NAME: &str = "cryptocrate.toml";

/// CryptoCrate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default compression level (1-21)
    #[serde(default = "default_compression_level")]
    pub compression_level: i32,

    /// Enable compression by default
    #[serde(default)]
    pub compress_by_default: bool,

    /// Default output directory
    #[serde(default)]
    pub default_output_dir: Option<String>,

    /// Confirm before overwriting files
    #[serde(default = "default_true")]
    pub confirm_overwrite: bool,

    /// Show detailed progress information
    #[serde(default = "default_true")]
    pub show_detailed_progress: bool,

    /// Argon2 memory cost in KB
    #[serde(default = "default_argon2_memory")]
    pub argon2_memory_kb: u32,

    /// Argon2 time cost (iterations)
    #[serde(default = "default_argon2_time")]
    pub argon2_time_cost: u32,

    /// Argon2 parallelism
    #[serde(default = "default_argon2_parallelism")]
    pub argon2_parallelism: u32,
}

fn default_compression_level() -> i32 {
    3
}

fn default_true() -> bool {
    true
}

fn default_argon2_memory() -> u32 {
    65536 // 64 MB
}

fn default_argon2_time() -> u32 {
    3
}

fn default_argon2_parallelism() -> u32 {
    4
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compression_level: default_compression_level(),
            compress_by_default: false,
            default_output_dir: None,
            confirm_overwrite: true,
            show_detailed_progress: true,
            argon2_memory_kb: default_argon2_memory(),
            argon2_time_cost: default_argon2_time(),
            argon2_parallelism: default_argon2_parallelism(),
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| CrateError::Io(e))?;
        
        let config: Config = toml::from_str(&content)
            .map_err(|e| CrateError::InvalidFormat(format!("Invalid config file: {}", e)))?;
        
        Ok(config)
    }

    /// Load configuration from default locations
    /// Priority: ./cryptocrate.toml > ~/.config/cryptocrate/config.toml
    pub fn load_default() -> Result<Self> {
        // Try current directory first
        let local_config = PathBuf::from(CONFIG_FILE_NAME);
        if local_config.exists() {
            return Self::load(&local_config);
        }

        // Try user config directory
        if let Some(home) = dirs::home_dir() {
            let user_config = home.join(".config").join("cryptocrate").join("config.toml");
            if user_config.exists() {
                return Self::load(&user_config);
            }
        }

        // Return default config if no file found
        Ok(Config::default())
    }

    /// Save configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| CrateError::InvalidFormat(format!("Failed to serialize config: {}", e)))?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Get the default user config path
    pub fn default_user_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| {
            home.join(".config").join("cryptocrate").join("config.toml")
        })
    }

    /// Generate a sample configuration file content
    pub fn sample() -> String {
        r#"# CryptoCrate Configuration File
# This file configures default behavior for the cryptocrate tool.

# Default compression level (1-21, higher = better compression but slower)
compression_level = 3

# Enable compression by default (can be overridden with --compress flag)
compress_by_default = false

# Default output directory (leave empty to use same directory as input)
# default_output_dir = "/path/to/encrypted"

# Confirm before overwriting existing files
confirm_overwrite = true

# Show detailed progress information
show_detailed_progress = true

# Argon2 key derivation parameters (advanced users only)
# Higher values = more secure but slower
argon2_memory_kb = 65536  # 64 MB
argon2_time_cost = 3       # iterations
argon2_parallelism = 4     # threads
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.compression_level, 3);
        assert!(!config.compress_by_default);
        assert!(config.confirm_overwrite);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");

        let config = Config::default();
        config.save(&config_path).unwrap();

        let loaded = Config::load(&config_path).unwrap();
        assert_eq!(loaded.compression_level, config.compression_level);
    }

    #[test]
    fn test_sample_config() {
        let sample = Config::sample();
        assert!(sample.contains("compression_level"));
        assert!(sample.contains("argon2"));
    }
}
