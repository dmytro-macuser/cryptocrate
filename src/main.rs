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
mod keyfile;
mod metadata;
mod secure_delete;
mod streaming;

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
use keyfile::{combine_password_and_keyfile, generate_keyfile, read_keyfile, DEFAULT_KEYFILE_SIZE};
use secure_delete::{secure_delete, SecureDeleteMode};
use streaming::{decrypt_file_streaming, encrypt_file_streaming, should_use_streaming};

#[derive(Parser)]
#[command(name = "cryptocrate")]
#[command(author = "Dmytro Vlasiuk")]
#[command(version = "1.0.0")]
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

        /// Key file for encryption (can be combined with password)
        #[arg(short, long)]
        keyfile: Option<PathBuf>,

        /// Securely delete original files after encryption
        #[arg(long)]
        delete: bool,

        /// Secure deletion mode (quick, standard, paranoid)
        #[arg(long, default_value = "standard")]
        delete_mode: String,

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

        /// Key file for decryption (if used during encryption)
        #[arg(short, long)]
        keyfile: Option<PathBuf>,

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
    /// Generate a new key file
    Keygen {
        /// Output path for the key file
        #[arg(value_name = "PATH")]
        output: PathBuf,

        /// Key file size in bytes (default: 4096)
        #[arg(short, long)]
        size: Option<usize>,
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

/// Parse secure delete mode
fn parse_delete_mode(mode_str: &str) -> SecureDeleteMode {
    match mode_str.to_lowercase().as_str() {
        "quick" | "q" => SecureDeleteMode::Quick,
        "standard" | "s" => SecureDeleteMode::Standard,
        "paranoid" | "p" => SecureDeleteMode::Paranoid,
        _ => SecureDeleteMode::Standard,
    }
}

/// Get password with optional keyfile
fn get_password_with_keyfile(
    password_opt: Option<String>,
    keyfile_opt: Option<PathBuf>,
    for_encryption: bool,
) -> Result<String> {
    let keyfile_hash = if let Some(keyfile_path) = keyfile_opt {
        println!("🔑 Reading key file: {}", keyfile_path.display());
        Some(read_keyfile(&keyfile_path)?)
    } else {
        None
    };

    let password = match password_opt {
        Some(p) => p,
        None => {
            if keyfile_hash.is_some() {
                println!("\n💡 Tip: Key file detected. You can optionally add a password for two-factor security.");
                println!("   Press Enter to skip password (key file only).");
            }

            if for_encryption {
                prompt_password_with_confirm("Enter password (or press Enter to skip)")?
            } else {
                prompt_password("Enter password (or press Enter if using key file only)")?
            }
        }
    };

    // Combine password and keyfile if both are provided
    if let Some(kf_hash) = keyfile_hash {
        if password.is_empty() {
            // Key file only - use keyfile hash as "password"
            Ok(hex::encode(kf_hash))
        } else {
            // Both password and keyfile - combine them
            let combined = combine_password_and_keyfile(&password, &kf_hash);
            Ok(hex::encode(combined))
        }
    } else {
        // Password only
        if password.is_empty() {
            anyhow::bail!("Password cannot be empty!\n\n💡 Tip: Provide either a password, a key file, or both.");
        }
        Ok(password)
    }
}

/// Enhanced error messages with suggestions
fn handle_error(err: anyhow::Error) {
    eprintln!("❌ Error: {}", err);

    let err_str = err.to_string();

    // Provide helpful suggestions based on error type
    if err_str.contains("Password") || err_str.contains("password") {
        eprintln!("\n💡 Tip: Make sure you're using the correct password and/or key file.");
        eprintln!("   If you used a key file during encryption, you must use the same file for decryption.");
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
    keyfile: Option<PathBuf>,
    delete_originals: bool,
    delete_mode_str: String,
    yes: bool,
    config: &Config,
) -> Result<()> {
    // Validate all paths exist
    for path in &paths {
        if !path.exists() {
            anyhow::bail!(
                "Path not found: {}\n\n💡 Tip: Check your spelling and that the file/folder exists.",
                path.display()
            );
        }
    }

    // Warning about compression with streaming
    if compress {
        println!("💡 Note: Large files (>100 MB) cannot use compression due to streaming mode.");
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

    // Check for large files
    let large_file_count = all_files
        .iter()
        .filter(|f| f.size > streaming::STREAMING_THRESHOLD)
        .count();
    if large_file_count > 0 {
        println!(
            "📦 {} large file(s) detected - will use streaming mode (no compression)",
            large_file_count
        );
    }

    // Calculate total size
    let total_size: u64 = all_files.iter().map(|f| f.size).sum();
    let file_count = all_files.len();

    println!("\n📊 Encryption Summary:");
    println!("   Files: {}", file_count);
    println!("   Total size: {}", format_size(total_size));
    println!(
        "   Compression: {}",
        if compress { "✅ enabled" } else { "❌ disabled" }
    );
    if keyfile.is_some() {
        println!("   Key file: ✅ will be used");
    }
    if delete_originals {
        let mode = parse_delete_mode(&delete_mode_str);
        println!("   Secure delete: ✅ enabled ({:?} mode)", mode);
    }
    println!();

    // Confirm deletion if enabled
    if delete_originals && !yes {
        if !confirm(
            "⚠️  Original files will be PERMANENTLY deleted after encryption. Continue?",
            false,
        )? {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    // Get password (possibly combined with keyfile)
    let password = get_password_with_keyfile(password, keyfile, true)?;

    // Determine output directory
    let output_dir = output_dir.or_else(|| config.default_output_dir.as_ref().map(PathBuf::from));

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
    let delete_mode = parse_delete_mode(&delete_mode_str);

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

        // Encrypt the file (use streaming for large files)
        let use_streaming = should_use_streaming(&file_entry.path)?;
        let should_compress = compress && !use_streaming;

        let encrypt_result = if use_streaming {
            encrypt_file_streaming(&file_entry.path, &output_path, &password)
        } else {
            encrypt_file(&file_entry.path, &output_path, &password, should_compress)
        };

        match encrypt_result {
            Ok(_) => {
                total_original_size += file_entry.size;
                if let Ok(metadata) = fs::metadata(&output_path) {
                    total_encrypted_size += metadata.len();
                }

                // Securely delete original if requested
                if delete_originals {
                    if let Err(e) = secure_delete(&file_entry.path, delete_mode) {
                        if let Some(ref pb) = file_pb {
                            pb.finish_with_message(format!(
                                "⚠️  {} - Encrypted but failed to delete: {}",
                                filename, e
                            ));
                        }
                    } else {
                        if let Some(ref pb) = file_pb {
                            pb.finish_with_message(format!(
                                "✅ {} ({}) 🗜️ ",
                                filename,
                                format_size(file_entry.size)
                            ));
                        }
                    }
                } else {
                    if let Some(ref pb) = file_pb {
                        pb.finish_with_message(format!(
                            "✅ {} ({})",
                            filename,
                            format_size(file_entry.size)
                        ));
                    }
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

    if delete_originals {
        println!("   🗜️  Original files securely deleted");
    }

    Ok(())
}

/// Handle decryption command
fn handle_decrypt(
    paths: Vec<PathBuf>,
    output_dir: Option<PathBuf>,
    password: Option<String>,
    keyfile: Option<PathBuf>,
    yes: bool,
    config: &Config,
) -> Result<()> {
    // Validate all paths exist
    for path in &paths {
        if !path.exists() {
            anyhow::bail!(
                "File not found: {}\n\n💡 Tip: Make sure the .crat file exists.",
                path.display()
            );
        }
        if !path.is_file() {
            anyhow::bail!("Not a file: {}\n\n💡 Tip: Decrypt command only works with files, not directories.", path.display());
        }
    }

    let file_count = paths.len();
    println!("\n📊 Decryption Summary:");
    println!("   Files: {}", file_count);
    if keyfile.is_some() {
        println!("   Key file: ✅ will be used");
    }
    println!();

    // Get password (possibly combined with keyfile)
    let password = get_password_with_keyfile(password, keyfile, false)?;

    // Determine output directory
    let output_dir = output_dir.or_else(|| config.default_output_dir.as_ref().map(PathBuf::from));

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

        // Check if we should use streaming
        let file_size = fs::metadata(path)?.len();
        let use_streaming = file_size > streaming::STREAMING_THRESHOLD;

        // Decrypt the file
        let decrypt_result = if use_streaming {
            decrypt_file_streaming(path, &temp_output, &password)
        } else {
            decrypt_file(path, &temp_output, &password)
        };

        match decrypt_result {
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

/// Handle keygen command
fn handle_keygen(output: PathBuf, size: Option<usize>) -> Result<()> {
    let size = size.unwrap_or(DEFAULT_KEYFILE_SIZE);

    println!("\n🔑 Generating key file...");
    println!("   Path: {}", output.display());
    println!("   Size: {} bytes ({} KB)", size, size / 1024);

    if output.exists() {
        if !confirm(
            &format!("Key file already exists at {:?}. Overwrite?", output),
            false,
        )? {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

    generate_keyfile(&output, Some(size))?;

    println!("\n✅ Key file generated successfully!");
    println!("\n⚠️  IMPORTANT:");
    println!("   - Keep this key file SAFE and SECURE");
    println!("   - Make a BACKUP copy in a safe location");
    println!("   - Anyone with this file can decrypt your data");
    println!("   - If you lose it, you CANNOT decrypt your files");
    println!("\n💡 Usage:");
    println!("   cryptocrate encrypt file.txt --keyfile {}", output.display());
    println!("   cryptocrate decrypt file.txt.crat --keyfile {}", output.display());

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
                if !confirm(
                    &format!("Config file already exists at {:?}. Overwrite?", path),
                    false,
                )? {
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
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to open editor: {}. Try setting $EDITOR environment variable.",
                        e
                    )
                })?;
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
            keyfile,
            delete,
            delete_mode,
            yes,
        } => handle_encrypt(
            paths,
            compress,
            output,
            password,
            keyfile,
            delete,
            delete_mode,
            yes,
            &config,
        ),
        Commands::Decrypt {
            paths,
            output,
            password,
            keyfile,
            yes,
        } => handle_decrypt(paths, output, password, keyfile, yes, &config),
        Commands::Inspect { paths } => handle_inspect(paths),
        Commands::Keygen { output, size } => handle_keygen(output, size),
        Commands::Config { action } => handle_config(action),
    };

    if let Err(e) = result {
        handle_error(e);
        std::process::exit(1);
    }
}
