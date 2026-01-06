//! CryptoCrate - A fast, user-friendly file and folder encryption tool
//!
//! This tool uses AES-256-GCM for encryption and Argon2id for key derivation,
//! providing strong security while remaining easy to use for beginners.

mod compression;
mod crypto;
mod error;
mod file_handler;
mod format;
mod metadata;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use compression::compression_ratio;
use crypto::{decrypt_file, encrypt_file};
use file_handler::collect_files;

#[derive(Parser)]
#[command(name = "cryptocrate")]
#[command(author = "Dmytro Vlasiuk")]
#[command(version = "0.2.0")]
#[command(about = "🔐 A fast, user-friendly file and folder encryption tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Encrypt files or folders
    Encrypt {
        /// Paths to files or folders to encrypt (supports multiple)
        #[arg(value_name = "PATH", required = true)]
        paths: Vec<PathBuf>,

        /// Enable compression before encryption
        #[arg(short, long)]
        compress: bool,

        /// Output directory (default: same as input)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Password for encryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Decrypt files or folders
    Decrypt {
        /// Paths to encrypted files (.crat)
        #[arg(value_name = "PATH", required = true)]
        paths: Vec<PathBuf>,

        /// Output directory (default: same as input)
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

/// Handle encryption command
fn handle_encrypt(
    paths: Vec<PathBuf>,
    compress: bool,
    output_dir: Option<PathBuf>,
    password: Option<String>,
) -> Result<()> {
    // Validate all paths exist
    for path in &paths {
        if !path.exists() {
            anyhow::bail!("Path not found: {}", path.display());
        }
    }

    // Collect all files from all paths
    let mut all_files = Vec::new();
    for path in &paths {
        let files = collect_files(path, None)?;
        all_files.extend(files);
    }

    if all_files.is_empty() {
        println!("⚠️  No files found to encrypt!");
        return Ok(());
    }

    // Calculate total size
    let total_size: u64 = all_files.iter().map(|f| f.size).sum();
    let file_count = all_files.len();

    println!("\n📊 Encryption Summary:");
    println!("   Files: {}", file_count);
    println!("   Total size: {}", format_size(total_size));
    println!("   Compression: {}", if compress { "enabled" } else { "disabled" });
    println!();

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

    // Setup progress
    let multi_progress = MultiProgress::new();
    let overall_pb = multi_progress.add(ProgressBar::new(file_count as u64));
    overall_pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} files ({msg})")
            .unwrap()
            .progress_chars("=>-"),
    );

    let start_time = Instant::now();
    let mut total_original_size = 0u64;
    let mut total_encrypted_size = 0u64;
    let mut success_count = 0;
    let mut error_count = 0;

    // Encrypt each file
    for (idx, file_entry) in all_files.iter().enumerate() {
        let file_pb = multi_progress.add(ProgressBar::new_spinner());
        file_pb.set_style(
            ProgressStyle::default_spinner()
                .template("  {spinner:.green} {msg}")
                .unwrap(),
        );

        let filename = file_entry.path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        file_pb.set_message(format!("{} ({})", filename, format_size(file_entry.size)));
        file_pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Determine output path
        let output_path = if let Some(ref out_dir) = output_dir {
            fs::create_dir_all(out_dir)?;
            out_dir.join(format!("{}.crat", filename))
        } else {
            file_entry.path.with_extension("crat")
        };

        // Encrypt the file
        match encrypt_file(&file_entry.path, &output_path, &password, compress) {
            Ok(_) => {
                total_original_size += file_entry.size;
                if let Ok(metadata) = fs::metadata(&output_path) {
                    total_encrypted_size += metadata.len();
                }
                file_pb.finish_with_message(format!("✅ {} ({})", filename, format_size(file_entry.size)));
                success_count += 1;
            }
            Err(e) => {
                file_pb.finish_with_message(format!("❌ {} - Error: {}", filename, e));
                error_count += 1;
            }
        }

        overall_pb.set_position((idx + 1) as u64);
        overall_pb.set_message(format!("✅ {} / ❌ {}", success_count, error_count));
    }

    let duration = start_time.elapsed();
    overall_pb.finish_with_message(format!("Done! ✅ {} / ❌ {}", success_count, error_count));

    // Print summary
    println!("\n🎉 Encryption Complete!");
    println!("   Success: {} files", success_count);
    if error_count > 0 {
        println!("   Errors: {} files", error_count);
    }
    println!("   Time: {:.2}s", duration.as_secs_f64());
    
    if compress && total_original_size > 0 {
        let ratio = compression_ratio(total_original_size as usize, total_encrypted_size as usize);
        println!("   Original size: {}", format_size(total_original_size));
        println!("   Encrypted size: {}", format_size(total_encrypted_size));
        println!("   Space saved: {:.1}%", ratio);
    }

    Ok(())
}

/// Handle decryption command
fn handle_decrypt(
    paths: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    password: Option<String>,
) -> Result<()> {
    // Validate all paths exist
    for path in &paths {
        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }
        if !path.is_file() {
            anyhow::bail!("Not a file: {}", path.display());
        }
    }

    let file_count = paths.len();
    println!("\n📊 Decryption Summary:");
    println!("   Files: {}", file_count);
    println!();

    // Get password
    let password = match password {
        Some(p) => p,
        None => prompt_password("Enter password: ")?,
    };

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty!");
    }

    // Create output directory if specified
    if let Some(ref out_dir) = output_dir {
        fs::create_dir_all(out_dir)?;
    }

    // Setup progress
    let multi_progress = MultiProgress::new();
    let overall_pb = multi_progress.add(ProgressBar::new(file_count as u64));
    overall_pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} files ({msg})")
            .unwrap()
            .progress_chars("=>-"),
    );

    let start_time = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    // Decrypt each file
    for (idx, path) in paths.iter().enumerate() {
        let file_pb = multi_progress.add(ProgressBar::new_spinner());
        file_pb.set_style(
            ProgressStyle::default_spinner()
                .template("  {spinner:.green} {msg}")
                .unwrap(),
        );

        let filename = path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        file_pb.set_message(filename.to_string());
        file_pb.enable_steady_tick(std::time::Duration::from_millis(100));

        // Determine output path (temporary)
        let temp_output = if let Some(ref out_dir) = output_dir {
            out_dir.join("temp_decrypt")
        } else {
            path.with_file_name("temp_decrypt")
        };

        // Decrypt the file
        match decrypt_file(path, &temp_output, &password) {
            Ok(metadata) => {
                // Move to final location with original filename
                let final_output = if let Some(ref out_dir) = output_dir {
                    out_dir.join(&metadata.filename)
                } else {
                    path.with_file_name(&metadata.filename)
                };

                if let Err(e) = fs::rename(&temp_output, &final_output) {
                    file_pb.finish_with_message(format!("❌ {} - Error: {}", filename, e));
                    error_count += 1;
                } else {
                    let compressed_msg = if metadata.is_compressed { " (compressed)" } else { "" };
                    file_pb.finish_with_message(format!("✅ {} → {}{}", filename, metadata.filename, compressed_msg));
                    success_count += 1;
                }
            }
            Err(e) => {
                file_pb.finish_with_message(format!("❌ {} - Error: {}", filename, e));
                error_count += 1;
                // Clean up temp file if it exists
                let _ = fs::remove_file(&temp_output);
            }
        }

        overall_pb.set_position((idx + 1) as u64);
        overall_pb.set_message(format!("✅ {} / ❌ {}", success_count, error_count));
    }

    let duration = start_time.elapsed();
    overall_pb.finish_with_message(format!("Done! ✅ {} / ❌ {}", success_count, error_count));

    // Print summary
    println!("\n🎉 Decryption Complete!");
    println!("   Success: {} files", success_count);
    if error_count > 0 {
        println!("   Errors: {} files", error_count);
    }
    println!("   Time: {:.2}s", duration.as_secs_f64());

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encrypt {
            paths,
            compress,
            output,
            password,
        } => handle_encrypt(paths, compress, output, password)?,
        Commands::Decrypt {
            paths,
            output,
            password,
        } => handle_decrypt(paths, output, password)?,
    }

    Ok(())
}
