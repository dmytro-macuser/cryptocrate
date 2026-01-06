//! CryptoCrate - A fast, user-friendly file and folder encryption tool
//!
//! This tool uses AES-256-GCM for encryption and Argon2id for key derivation,
//! providing strong security while remaining easy to use for beginners.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cryptocrate")]
#[command(author = "Dmytro Vlasiuk")]
#[command(version = "0.1.0")]
#[command(about = "🔐 A fast, user-friendly file and folder encryption tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt a file or folder
    Encrypt {
        /// Path to file or folder to encrypt
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Enable compression before encryption
        #[arg(short, long)]
        compress: bool,

        /// Output path (default: <input>.crat)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Decrypt a file or folder
    Decrypt {
        /// Path to encrypted file (.crat)
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt {
            path,
            compress,
            output,
        } => {
            println!("🔒 Encrypting: {:?}", path);
            println!("   Compression: {}", if compress { "enabled" } else { "disabled" });
            if let Some(out) = output {
                println!("   Output: {:?}", out);
            }
            println!("\n⚠️  Implementation coming soon!");
            println!("   This will use AES-256-GCM encryption with Argon2id key derivation.");
        }
        Commands::Decrypt { path, output } => {
            println!("🔓 Decrypting: {:?}", path);
            if let Some(out) = output {
                println!("   Output: {:?}", out);
            }
            println!("\n⚠️  Implementation coming soon!");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert_eq!(2 + 2, 4);
    }
}
