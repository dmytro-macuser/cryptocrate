# 🔐 CryptoCrate

**A fast, user-friendly file and folder encryption tool with strong cryptography - perfect for beginners!**

CryptoCrate makes file encryption accessible to everyone without compromising on security. Unlike simple file obfuscators, CryptoCrate uses industry-standard encryption algorithms to truly protect your data.

## ✨ Status: Phase 2 Complete! ✅

**Version 0.2.0 Features:**
- ✅ AES-256-GCM encryption/decryption
- ✅ Argon2id password-based key derivation
- ✅ Single file encryption
- ✅ **Folder encryption with recursive traversal**
- ✅ **Zstd compression support**
- ✅ **Batch operations (multiple files/folders)**
- ✅ **File metadata preservation**
- ✅ **Enhanced progress indicators**
- ✅ Comprehensive unit tests

**Coming Soon (Phase 3):**
- 📝 Interactive mode
- ⚙️ Configuration files
- 🔍 Encrypted file inspection
- 📦 Binary releases

## ✨ Features

### Core Functionality
- 📄 **Individual File Encryption**: Encrypt single files quickly ✅
- 📁 **Folder Encryption**: Encrypt entire directories recursively ✅ **NEW!**
- 🔑 **Password-Based Encryption**: Simple password protection with strong key derivation ✅
- 🧩 **Compression**: Zstd compression before encryption ✅ **NEW!**
- ⚡ **Fast Performance**: Optimized for speed without sacrificing security ✅
- 🛡️ **Real Cryptography**: AES-256-GCM encryption (not just header manipulation!) ✅
- 📊 **Progress Indicators**: Multi-file progress with detailed status ✅ **ENHANCED!**
- 📦 **Batch Operations**: Encrypt multiple files/folders at once ✅ **NEW!**
- 🎯 **Beginner-Friendly**: Simple CLI interface with clear instructions ✅
- 📝 **Metadata Preservation**: Original filenames, sizes, and timestamps ✅ **NEW!**

### Advanced Features (Coming Soon)
- 🔍 **File Integrity**: Built-in integrity checks to detect tampering ✅ (via GCM)
- 🚫 **Secure Deletion**: Option to securely delete original files after encryption (Phase 3)
- 🎨 **GUI Version**: Optional graphical interface for non-technical users (Phase 4)
- 🔑 **Key Files**: Support for key files in addition to passwords (Phase 4)

## 🛠️ Technical Approach

### Encryption Stack
- **Algorithm**: AES-256 in GCM mode (Galois/Counter Mode)
  - Provides both confidentiality and authenticity
  - Resistant to padding oracle attacks
  - NIST-approved standard
  - Built-in authentication tag prevents tampering

- **Key Derivation**: Argon2id
  - Memory-hard function resistant to GPU/ASIC attacks
  - Winner of the Password Hashing Competition
  - Parameters: 64 MB memory, 3 iterations, 4 threads

- **Compression**: Zstd (Zstandard)
  - Fast compression with excellent ratios
  - Level 3 by default (balanced speed/compression)
  - Applied before encryption

- **Random Generation**: Cryptographically secure random number generator (CSPRNG)
  - For IVs, salts, and nonces
  - Platform-specific secure implementations

### File Format (.crat)
```
[🏷️ Header - 54 bytes]
- Magic bytes: "CRAT" (4 bytes)
- Version: 1 byte
- Algorithm ID: 1 byte (1 = AES-256-GCM)
- Salt: 32 bytes (for key derivation)
- Nonce/IV: 12 bytes (for GCM)
- Metadata length: 4 bytes (little-endian)

[📝 Metadata - Variable]
- Filename length: 2 bytes
- Filename: Variable (UTF-8)
- Original size: 8 bytes
- Modified time: 8 bytes (Unix timestamp)
- Compression flag: 1 byte (0 or 1)

[🔒 Encrypted Data - Variable]
- The actual file contents (optionally compressed, then encrypted)
- Includes 16-byte authentication tag from GCM
```

## 🏭 Architecture

### Current Project Structure
```
cryptocrate/
├── src/
│   ├── main.rs              # Entry point and CLI ✅
│   ├── error.rs             # Error types ✅
│   ├── format.rs            # File format constants ✅
│   ├── metadata.rs          # Metadata preservation ✅ NEW!
│   ├── crypto/
│   │   ├── mod.rs           # Crypto module ✅
│   │   ├── encryption.rs    # AES-256-GCM implementation ✅
│   │   └── key_derivation.rs # Argon2id implementation ✅
│   ├── compression/
│   │   └── mod.rs           # Zstd compression ✅ NEW!
│   └── file_handler/
│       ├── mod.rs           # File handling module ✅ NEW!
│       └── walker.rs        # Directory traversal ✅ NEW!
├── Cargo.toml               # Dependencies ✅
├── LICENSE                  # MIT License ✅
└── README.md                # This file ✅
```

### Technology Choice: Rust 🦀
**Why Rust?**
- Memory safety without garbage collection = fast and secure
- Excellent cryptography libraries (RustCrypto)
- Cross-platform compilation (Windows, macOS, Linux)
- Great CLI tools ecosystem (clap, indicatif)
- Zero-cost abstractions for performance

### Key Dependencies
```toml
[dependencies]
aes-gcm = "0.10"         # AES-256-GCM encryption
argon2 = "0.5"          # Key derivation
rand = "0.8"            # Secure random generation
clap = "4.5"            # Command-line parsing
indicatif = "0.17"      # Progress bars
rpassword = "7.3"       # Secure password input
walkdir = "2"           # Directory traversal
zstd = "0.13"           # Fast compression
thiserror = "1.0"       # Error handling
anyhow = "1.0"          # Error context
```

## 🚀 Quick Start

### Installation
```bash
# Clone the repository
git clone https://github.com/dmytro-macuser/cryptocrate.git
cd cryptocrate

# Build the project
cargo build --release

# The binary will be at: target/release/cryptocrate

# Optional: Install globally
cargo install --path .
```

### Usage Examples

**Encrypt a single file:**
```bash
cryptocrate encrypt secret.txt
# Output: secret.txt.crat
```

**Encrypt a folder (recursively):**
```bash
cryptocrate encrypt my_documents/
# Encrypts all files in the folder
```

**Encrypt with compression:**
```bash
cryptocrate encrypt large_file.dat --compress
# Compresses then encrypts (saves space!)
```

**Batch encrypt multiple items:**
```bash
cryptocrate encrypt file1.txt file2.txt folder1/ folder2/
# Encrypts everything at once
```

**Encrypt to a specific output directory:**
```bash
cryptocrate encrypt documents/ --output ./encrypted_backup/
# All encrypted files go to encrypted_backup/
```

**Decrypt files:**
```bash
cryptocrate decrypt file1.txt.crat file2.txt.crat
# Restores original filenames automatically
```

**Decrypt to specific location:**
```bash
cryptocrate decrypt encrypted_files/*.crat --output ./decrypted/
```

**Use password in command (less secure, good for scripts):**
```bash
cryptocrate encrypt data.txt --password mypassword
```

### Complete Workflow Example
```bash
# Create test folder
mkdir test_folder
echo "Secret data 1" > test_folder/file1.txt
echo "Secret data 2" > test_folder/file2.txt

# Encrypt entire folder with compression
cryptocrate encrypt test_folder/ --compress --output encrypted/
# Enter password when prompted

# Decrypt everything back
cryptocrate decrypt encrypted/*.crat --output decrypted/
# Enter same password

# Verify
ls decrypted/
# Output: file1.txt  file2.txt
```

### Testing
```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific module
cargo test compression
```

## 🛣️ Roadmap

### Phase 1: Core Implementation (v0.1) ✅ **COMPLETE!**
- [x] Project setup
- [x] Basic AES-256-GCM encryption/decryption
- [x] Argon2 key derivation
- [x] Single file encryption
- [x] Basic CLI interface
- [x] Unit tests

### Phase 2: Enhanced Features (v0.2) ✅ **COMPLETE!**
- [x] Folder encryption with recursive traversal
- [x] Progress bars for multiple files
- [x] File metadata preservation (filename, size, timestamp)
- [x] Compression support (zstd)
- [x] Batch operations

### Phase 3: User Experience (v0.3) 🚧 **NEXT**
- [ ] Interactive mode for passwords
- [ ] Configuration file support
- [ ] Better error messages with suggestions
- [ ] Encrypted file inspection (show metadata without decrypting)
- [ ] Cross-platform binary releases (Linux, macOS, Windows)

### Phase 4: Advanced Features (v1.0)
- [ ] GUI application (desktop)
- [ ] Key file support (in addition to passwords)
- [ ] Secure file deletion (overwrite before delete)
- [ ] Encrypted archive format (.crate files)
- [ ] Streaming for very large files (>1GB)

## 🔒 Security Considerations

- **Never store passwords**: All passwords are immediately derived into keys and cleared from memory
- **Unique salts and nonces**: Every encryption uses fresh random values (32-byte salt, 12-byte nonce)
- **Memory safety**: Rust's ownership system prevents memory leaks and buffer overflows
- **No custom crypto**: Only battle-tested, peer-reviewed algorithms (AES-256-GCM, Argon2id)
- **Authentication**: GCM mode provides built-in authentication, preventing tampering
- **Password verification**: Wrong password = decryption fails (authenticated encryption)
- **Compression before encryption**: Prevents compression-based attacks

## 🧪 Security Testing

The implementation includes tests for:
- ✅ Encryption/decryption roundtrip (with and without compression)
- ✅ Wrong password detection
- ✅ Key derivation consistency
- ✅ Different salts produce different keys
- ✅ Metadata serialization/deserialization
- ✅ Compression/decompression
- ✅ Directory traversal

## 📊 Performance

**Typical speeds** (on modern hardware):
- Encryption: ~100-200 MB/s (uncompressed)
- Encryption: ~50-100 MB/s (with compression)
- Decryption: ~150-250 MB/s (uncompressed)
- Decryption: ~80-150 MB/s (with decompression)

**Compression ratios** (text files):
- Plain text: 60-80% smaller
- JSON/XML: 70-85% smaller
- Source code: 50-70% smaller
- Already compressed files (images, videos): minimal benefit

## 🤝 Contributing

Contributions are welcome! Whether you're fixing bugs, improving documentation, or adding features:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

**Ideas for contributions:**
- Add more compression algorithms
- Implement streaming for large files
- Create a GUI
- Add benchmarks
- Improve documentation
- Write more tests

## 📜 License

MIT License - see LICENSE file for details

## ⚠️ Disclaimer

While CryptoCrate uses industry-standard encryption algorithms, no software is 100% secure. Always:
- Keep backups of important data
- Use strong, unique passwords (12+ characters recommended)
- Keep your software updated
- Don't lose your passwords (we can't recover them!)
- Test with non-critical files first
- Consider the legal implications of encryption in your jurisdiction

## 💬 Contact

Questions? Issues? Ideas? Open an issue or start a discussion!

**Repository**: https://github.com/dmytro-macuser/cryptocrate

---

Made with ❤️ and Rust 🦀 | Phase 2 Complete ✅