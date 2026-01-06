//! CryptoCrate - A fast, user-friendly file and folder encryption tool
//!
//! This tool uses AES-256-GCM for encryption and Argon2id for key derivation,
//! providing strong security while remaining easy to use for beginners.

mod crypto;
mod error;
mod format;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::path::PathBuf;

use crypto::{decrypt_file, encrypt_file};

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

        /// Password for encryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Decrypt a file or folder
    Decrypt {
        /// Path to encrypted file (.crat)
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Output path (default: original filename)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Password for decryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
}

/// Prompt for password securely (without echo)
fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let password = rpassword::read_password()?;
    Ok(password)
}

/// Handle encryption command
fn handle_encrypt(
    path: PathBuf,
    compress: bool,
    output: Option<PathBuf>,
    password: Option<String>,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    if path.is_dir() {
        println!("⚠️  Folder encryption not yet implemented (coming in Phase 2)!");
        println!("   For now, please encrypt individual files.");
        return Ok(());
    }

    if compress {
        println!("⚠️  Compression not yet implemented (coming in Phase 2)!");
        println!("   Encrypting without compression...");
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        let mut p = path.clone();
        let new_name = format!(
            "{}.crat",
            p.file_name().unwrap_or_default().to_string_lossy()
        );
        p.set_file_name(new_name);
        p
    });

    // Get password
    let password = match password {
        Some(p) => p,
        None => {
            let pass1 = prompt_password("Enter password: ")?;
            let pass2 = prompt_password("Confirm password: ")?;
            if pass1 != pass2 {
                anyhow::bail!("Passwords do not match!");
            }
            pass1
        }
    };

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty!");
    }

    // Create progress bar
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Encrypting {}...", path.display()));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Encrypt the file
    match encrypt_file(&path, &output_path, &password) {
        Ok(_) => {
            spinner.finish_with_message(format!("✅ Encrypted: {}", output_path.display()));
            println!("   Algorithm: AES-256-GCM");
            println!("   Key derivation: Argon2id");
        }
        Err(e) => {
            spinner.finish_with_message("❌ Encryption failed");
            anyhow::bail!("Encryption error: {}", e);
        }
    }

    Ok(())
}

/// Handle decryption command
fn handle_decrypt(
    path: PathBuf,
    output: Option<PathBuf>,
    password: Option<String>,
) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    // Get password
    let password = match password {
        Some(p) => p,
        None => prompt_password("Enter password: ")?,
    };

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty!");
    }

    // Create progress bar
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Decrypting {}...", path.display()));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    // Determine output path if not specified
    let temp_output = output.is_none();
    let output_path = output.unwrap_or_else(|| {
        path.with_file_name("decrypted_output")
    });

    // Decrypt the file
    match decrypt_file(&path, &output_path, &password) {
        Ok(original_filename) => {
            // If no output was specified, rename to original filename
            let final_path = if temp_output {
                let final_p = path.with_file_name(&original_filename);
                std::fs::rename(&output_path, &final_p)?;
                final_p
            } else {
                output_path
            };

            spinner.finish_with_message(format!("✅ Decrypted: {}", final_path.display()));
            println!("   Original name: {}", original_filename);
        }
        Err(e) => {
            spinner.finish_with_message("❌ Decryption failed");
            anyhow::bail!("Decryption error: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt {
            path,
            compress,
            output,
            password,
        } => handle_encrypt(path, compress, output, password)?,
        Commands::Decrypt {
            path,
            output,
            password,
        } => handle_decrypt(path, output, password)?,
    }

    Ok(())
}
