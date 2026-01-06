//! CryptoCrate - A fast, user-friendly file and folder encryption tool
//!
//! This tool uses AES-256-GCM for encryption and Argon2id for key derivation,
//! providing strong security while remaining easy to use for beginners.

mod compression;
mod config;
mod crypto;
mod error;
mod file_handler;
mod format;
mod inspect;
mod interactive;
mod metadata;

use anyhow::Result;
use clap::{Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use compression::compression_ratio;
use config::Config;
use crypto::{decrypt_file, encrypt_file};
use file_handler::collect_files;
use inspect::inspect_file;
use interactive::{confirm, prompt_password, prompt_password_with_confirm};

#[derive(Parser)]
#[command(name = "cryptocrate")]
#[command(author = "Dmytro Vlasiuk")]
#[command(version = "0.3.0")]
#[command(about = "🔐 A fast, user-friendly file and folder encryption tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to config file (default: auto-detect)
    #[arg(long, global = true)]
    config: Option<PathBuf>,
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

        /// Output directory (default: same as input or from config)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Password for encryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Decrypt files or folders
    Decrypt {
        /// Paths to encrypted files (.crat)
        #[arg(value_name = "PATH", required = true)]
        paths: Vec<PathBuf>,

        /// Output directory (default: same as input or from config)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Password for decryption (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// Inspect encrypted file metadata without decrypting
    Inspect {
        /// Paths to encrypted files (.crat) to inspect
        #[arg(value_name = "PATH", required = true)]
        paths: Vec<PathBuf>,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Initialize a new configuration file
    Init {
        /// Create in current directory instead of user config
        #[arg(short, long)]
        local: bool,
    },
    /// Edit configuration file
    Edit,
    /// Show configuration file path
    Path,
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

/// Enhanced error messages with suggestions
fn handle_error(err: anyhow::Error) {
    eprintln!("❌ Error: {}", err);
    
    let err_str = err.to_string();
    
    // Provide helpful suggestions based on error type
    if err_str.contains("Password") || err_str.contains("password") {
        eprintln!("\n💡 Tip: Make sure you're using the correct password.");
        eprintln!("   Passwords are case-sensitive and must match exactly.");
    } else if err_str.contains("not found") || err_str.contains("No such file") {
        eprintln!("\n💡 Tip: Check that the file path is correct and the file exists.");
        eprintln!("   Use 'ls' or 'dir' to list files in the current directory.");
    } else if err_str.contains("Permission denied") {
        eprintln!("\n💡 Tip: You don't have permission to access this file.");
        eprintln!("   Try running with appropriate permissions or check file ownership.");
    } else if err_str.contains("Invalid") || err_str.contains("Not a valid") {
        eprintln!("\n💡 Tip: This doesn't appear to be a valid CryptoCrate file.");
        eprintln!("   Make sure you're trying to decrypt a .crat file created by CryptoCrate.");
    } else if err_str.contains("disk") || err_str.contains("space") {
        eprintln!("\n💡 Tip: You may be running out of disk space.");
        eprintln!("   Free up some space and try again.");
    }
}

/// Handle encryption command
fn handle_encrypt(
    paths: Vec<PathBuf>,
    compress: bool,
    output_dir: Option<PathBuf>,
    password: Option<String>,
    yes: bool,
    config: &Config,
) -> Result<()> {
    // Validate all paths exist
    for path in &paths {
        if !path.exists() {
            anyhow::bail!("Path not found: {}\n\n💡 Tip: Check your spelling and that the file/folder exists.", path.display());
        }
    }

    // Use compression from config if not specified
    let compress = compress || config.compress_by_default;

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

    // Check for existing encrypted files
    if !yes && config.confirm_overwrite {
        let output_dir_ref = output_dir.as_ref().or(config.default_output_dir.as_ref().map(|s| Path::new(s)));
        let mut overwrite_count = 0;
        
        for file_entry in &all_files {
            let filename = file_entry.path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            let output_path = if let Some(out_dir) = output_dir_ref {
                out_dir.join(format!("{}.crat", filename))
            } else {
                file_entry.path.with_extension("crat")
            };
            
            if output_path.exists() {
                overwrite_count += 1;
            }
        }
        
        if overwrite_count > 0 {
            if !confirm(&format!("\n⚠️  {} encrypted file(s) will be overwritten. Continue?", overwrite_count), false)? {
                println!("Operation cancelled.");
                return Ok(());
            }
        }
    }

    // Calculate total size
    let total_size: u64 = all_files.iter().map(|f| f.size).sum();
    let file_count = all_files.len();

    println!("\n📊 Encryption Summary:");
    println!("   Files: {}", file_count);
    println!("   Total size: {}", format_size(total_size));
    println!("   Compression: {}", if compress { "✅ enabled" } else { "❌ disabled" });
    println!();

    // Get password
    let password = match password {
        Some(p) => p,
        None => prompt_password_with_confirm("Enter password")?,
    };

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty!\n\n💡 Tip: Use a strong password with at least 12 characters.");
    }

    // Determine output directory
    let output_dir = output_dir.or_else(|| {
        config.default_output_dir.as_ref().map(PathBuf::from)
    });

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
        let file_pb = if config.show_detailed_progress {
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("  {spinner:.green} {msg}")
                    .unwrap(),
            );
            Some(pb)
        } else {
            None
        };

        let filename = file_entry
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        if let Some(ref pb) = file_pb {
            pb.set_message(format!("{} ({})", filename, format_size(file_entry.size)));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
        }

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
                if let Some(ref pb) = file_pb {
                    pb.finish_with_message(format!(
                        "✅ {} ({})",
                        filename,
                        format_size(file_entry.size)
                    ));
                }
                success_count += 1;
            }
            Err(e) => {
                if let Some(ref pb) = file_pb {
                    pb.finish_with_message(format!("❌ {} - Error: {}", filename, e));
                }
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
        println!("   ⚠️  Errors: {} files", error_count);
    }
    println!("   Time: {:.2}s", duration.as_secs_f64());

    if compress && total_original_size > 0 {
        let ratio = compression_ratio(
            total_original_size as usize,
            total_encrypted_size as usize,
        );
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
    yes: bool,
    config: &Config,
) -> Result<()> {
    // Validate all paths exist
    for path in &paths {
        if !path.exists() {
            anyhow::bail!("File not found: {}\n\n💡 Tip: Make sure the .crat file exists.", path.display());
        }
        if !path.is_file() {
            anyhow::bail!("Not a file: {}\n\n💡 Tip: Decrypt command only works with files, not directories.", path.display());
        }
    }

    let file_count = paths.len();
    println!("\n📊 Decryption Summary:");
    println!("   Files: {}", file_count);
    println!();

    // Get password
    let password = match password {
        Some(p) => p,
        None => prompt_password("Enter password")?,
    };

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty!");
    }

    // Determine output directory
    let output_dir = output_dir.or_else(|| config.default_output_dir.as_ref().map(PathBuf::from));

    // Create output directory if specified
    if let Some(ref out_dir) = output_dir {
        fs::create_dir_all(out_dir)?;
    }

    // Check for existing files
    if !yes && config.confirm_overwrite {
        // We'll check as we go since we don't know original filenames yet
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
        let file_pb = if config.show_detailed_progress {
            let pb = multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("  {spinner:.green} {msg}")
                    .unwrap(),
            );
            Some(pb)
        } else {
            None
        };

        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("unknown");

        if let Some(ref pb) = file_pb {
            pb.set_message(filename.to_string());
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
        }

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

                // Check if file exists
                if final_output.exists() && !yes && config.confirm_overwrite {
                    if !confirm(
                        &format!("Overwrite existing file '{}'?", metadata.filename),
                        false,
                    )? {
                        let _ = fs::remove_file(&temp_output);
                        if let Some(ref pb) = file_pb {
                            pb.finish_with_message(format!("⏭️  {} - Skipped", filename));
                        }
                        continue;
                    }
                }

                if let Err(e) = fs::rename(&temp_output, &final_output) {
                    if let Some(ref pb) = file_pb {
                        pb.finish_with_message(format!("❌ {} - Error: {}", filename, e));
                    }
                    error_count += 1;
                } else {
                    let compressed_msg = if metadata.is_compressed {
                        " (was compressed)"
                    } else {
                        ""
                    };
                    if let Some(ref pb) = file_pb {
                        pb.finish_with_message(format!(
                            "✅ {} → {}{}",
                            filename, metadata.filename, compressed_msg
                        ));
                    }
                    success_count += 1;
                }
            }
            Err(e) => {
                if let Some(ref pb) = file_pb {
                    pb.finish_with_message(format!("❌ {} - Error: {}", filename, e));
                }
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
        println!("   ⚠️  Errors: {} files", error_count);
    }
    println!("   Time: {:.2}s", duration.as_secs_f64());

    Ok(())
}

/// Handle inspect command
fn handle_inspect(paths: Vec<PathBuf>) -> Result<()> {
    for (idx, path) in paths.iter().enumerate() {
        if idx > 0 {
            println!("\n{}", "=".repeat(60));
        }
        
        println!("\n🔍 Inspecting: {}\n", path.display());
        
        match inspect_file(path) {
            Ok(info) => {
                print!("{}", info.display());
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                if e.to_string().contains("Not a valid") {
                    eprintln!("\n💡 Tip: This file was not created by CryptoCrate.");
                }
            }
        }
    }
    
    Ok(())
}

/// Handle config command
fn handle_config(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = Config::load_default()?;
            let toml_str = toml::to_string_pretty(&config)?;
            println!("\n⚙️  Current Configuration:\n");
            println!("{}", toml_str);
        }
        ConfigAction::Init { local } => {
            let path = if local {
                PathBuf::from("cryptocrate.toml")
            } else {
                Config::default_user_config_path()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine user config directory"))?
            };

            if path.exists() {
                if !confirm(&format!("Config file already exists at {:?}. Overwrite?", path), false)? {
                    println!("Operation cancelled.");
                    return Ok(());
                }
            }

            let sample = Config::sample();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, sample)?;
            println!("✅ Configuration file created: {:?}", path);
            println!("\n💡 Edit this file to customize CryptoCrate behavior.");
        }
        ConfigAction::Edit => {
            let path = if PathBuf::from("cryptocrate.toml").exists() {
                PathBuf::from("cryptocrate.toml")
            } else if let Some(user_path) = Config::default_user_config_path() {
                if user_path.exists() {
                    user_path
                } else {
                    anyhow::bail!("No config file found. Run 'cryptocrate config init' first.");
                }
            } else {
                anyhow::bail!("No config file found. Run 'cryptocrate config init' first.");
            };

            // Try to open with default editor
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            println!("Opening config file with {}...", editor);
            std::process::Command::new(editor)
                .arg(&path)
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to open editor: {}. Try setting $EDITOR environment variable.", e))?;
        }
        ConfigAction::Path => {
            if PathBuf::from("cryptocrate.toml").exists() {
                println!("./cryptocrate.toml (local)");
            } else if let Some(user_path) = Config::default_user_config_path() {
                if user_path.exists() {
                    println!("{} (user)", user_path.display());
                } else {
                    println!("No config file found.");
                    println!("\n💡 Run 'cryptocrate config init' to create one.");
                }
            } else {
                println!("No config file found.");
            }
        }
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    // Load configuration
    let config = if let Some(config_path) = cli.config {
        Config::load(&config_path).unwrap_or_else(|e| {
            eprintln!("⚠️  Warning: Could not load config file: {}", e);
            eprintln!("   Using default configuration.");
            Config::default()
        })
    } else {
        Config::load_default().unwrap_or_default()
    };

    let result = match cli.command {
        Commands::Encrypt {
            paths,
            compress,
            output,
            password,
            yes,
        } => handle_encrypt(paths, compress, output, password, yes, &config),
        Commands::Decrypt {
            paths,
            output,
            password,
            yes,
        } => handle_decrypt(paths, output, password, yes, &config),
        Commands::Inspect { paths } => handle_inspect(paths),
        Commands::Config { action } => handle_config(action),
    };

    if let Err(e) = result {
        handle_error(e);
        std::process::exit(1);
    }
}
