# 🔐 CryptoCrate

**A fast, user-friendly file and folder encryption tool with strong cryptography - perfect for beginners!**

CryptoCrate makes file encryption accessible to everyone without compromising on security. Unlike simple file obfuscators, CryptoCrate uses industry-standard encryption algorithms to truly protect your data.

## ✨ Status: Phase 1 Complete! ✅

**Working Features:**
- ✅ AES-256-GCM encryption/decryption for single files
- ✅ Argon2id password-based key derivation
- ✅ Secure password prompting (hidden input)
- ✅ Progress indicators with spinners
- ✅ Original filename preservation in encrypted files
- ✅ Comprehensive unit tests

**Coming Soon (Phase 2):**
- 📁 Folder encryption
- 🧩 Compression support
- 📦 Batch operations

## ✨ Features

### Core Functionality
- 📄 **Individual File Encryption**: Encrypt single files quickly ✅ **WORKING**
- 🔑 **Password-Based Encryption**: Simple password protection with strong key derivation ✅ **WORKING**
- ⚡ **Fast Performance**: Optimized for speed without sacrificing security
- 🛡️ **Real Cryptography**: AES-256-GCM encryption (not just header manipulation!)
- 📊 **Progress Indicators**: See encryption/decryption progress in real-time ✅ **WORKING**
- 🎯 **Beginner-Friendly**: Simple CLI interface with clear instructions

### Advanced Features (Coming Soon)
- 📁 **Folder Encryption**: Encrypt entire directories with one command (Phase 2)
- 🧩 **Compression**: Automatic compression before encryption to save space (Phase 2)
- 🔍 **File Integrity**: Built-in integrity checks to detect tampering (Phase 2)
- 🚫 **Secure Deletion**: Option to securely delete original files after encryption (Phase 3)
- 📝 **Batch Operations**: Encrypt multiple files/folders at once (Phase 2)
- 🎨 **GUI Version**: Optional graphical interface for non-technical users (Phase 4)

## 🛠️ Technical Approach

### Encryption Stack
- **Algorithm**: AES-256 in GCM mode (Galois/Counter Mode)
  - Provides both confidentiality and authenticity
  - Resistant to padding oracle attacks
  - NIST-approved standard

- **Key Derivation**: Argon2id
  - Memory-hard function resistant to GPU/ASIC attacks
  - Winner of the Password Hashing Competition
  - Parameters: 64 MB memory, 3 iterations, 4 threads

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
- Original filename (UTF-8)

[🔒 Encrypted Data - Variable]
- The actual file contents (encrypted)
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
│   └── crypto/
│       ├── mod.rs           # Crypto module ✅
│       ├── encryption.rs    # AES-256-GCM implementation ✅
│       └── key_derivation.rs # Argon2id implementation ✅
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

**Encrypt a file (with password prompt):**
```bash
cryptocrate encrypt secret.txt
# You'll be prompted to enter and confirm a password
# Output: secret.txt.crat
```

**Encrypt with password in command (less secure, but convenient for testing):**
```bash
cryptocrate encrypt secret.txt --password mypassword
```

**Decrypt a file:**
```bash
cryptocrate decrypt secret.txt.crat
# You'll be prompted for the password
# Output: secret.txt (original filename restored)
```

**Decrypt to a specific location:**
```bash
cryptocrate decrypt secret.txt.crat --output /path/to/decrypted.txt
```

**Specify custom output for encryption:**
```bash
cryptocrate encrypt document.pdf --output secure_doc.crat
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Test the encryption roundtrip
echo "Hello, CryptoCrate!" > test.txt
cargo run --release -- encrypt test.txt
cargo run --release -- decrypt test.txt.crat
cat test.txt  # Original content restored!
```

## 🛣️ Roadmap

### Phase 1: Core Implementation (v0.1) ✅ **COMPLETE!**
- [x] Project setup
- [x] Basic AES-256-GCM encryption/decryption
- [x] Argon2 key derivation
- [x] Single file encryption
- [x] Basic CLI interface
- [x] Unit tests

### Phase 2: Enhanced Features (v0.2) 🚧 **IN PROGRESS**
- [ ] Folder encryption with recursive traversal
- [ ] Progress bars for large files (with percentages)
- [ ] File metadata preservation (timestamps, permissions)
- [ ] Compression support (zstd)
- [ ] Batch operations

### Phase 3: User Experience (v0.3)
- [ ] Interactive mode for passwords
- [ ] Configuration file support
- [ ] Better error messages
- [ ] Encrypted file inspection (show metadata without decrypting)
- [ ] Cross-platform binary releases

### Phase 4: Advanced Features (v1.0)
- [ ] GUI application
- [ ] Key file support (in addition to passwords)
- [ ] Secure file deletion
- [ ] Encrypted archive format (.crate files)
- [ ] File streaming for very large files

## 🔒 Security Considerations

- **Never store passwords**: All passwords are immediately derived into keys and cleared from memory
- **Unique salts and nonces**: Every encryption uses fresh random values (32-byte salt, 12-byte nonce)
- **Memory safety**: Rust's ownership system prevents memory leaks and buffer overflows
- **No custom crypto**: Only battle-tested, peer-reviewed algorithms (AES-256-GCM, Argon2id)
- **Authentication**: GCM mode provides built-in authentication, preventing tampering
- **Password verification**: Wrong password = decryption fails (authenticated encryption)

## 🧪 Security Testing

The implementation includes tests for:
- ✅ Encryption/decryption roundtrip
- ✅ Wrong password detection
- ✅ Key derivation consistency
- ✅ Different salts produce different keys

## 🤝 Contributing

Contributions are welcome! Whether you're fixing bugs, improving documentation, or adding features:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

MIT License - see LICENSE file for details

## ⚠️ Disclaimer

While CryptoCrate uses industry-standard encryption algorithms, no software is 100% secure. Always:
- Keep backups of important data
- Use strong, unique passwords (12+ characters recommended)
- Keep your software updated
- Don't lose your passwords (we can't recover them!)
- Test with non-critical files first

## 💬 Contact

Questions? Issues? Ideas? Open an issue or start a discussion!

---

Made with ❤️ and Rust 🦀 | Phase 1 Complete ✅