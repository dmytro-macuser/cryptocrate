# 🔐 CryptoCrate

**A fast, user-friendly file and folder encryption tool with strong cryptography - perfect for beginners!**

CryptoCrate makes file encryption accessible to everyone without compromising on security. Unlike simple file obfuscators, CryptoCrate uses industry-standard encryption algorithms to truly protect your data.

## ✨ Features

### Core Functionality
- 📁 **Folder Encryption**: Encrypt entire directories with one command
- 📄 **Individual File Encryption**: Encrypt single files quickly
- 🔑 **Password-Based Encryption**: Simple password protection with strong key derivation
- ⚡ **Fast Performance**: Optimized for speed without sacrificing security
- 🛡️ **Real Cryptography**: AES-256-GCM encryption (not just header manipulation!)
- 📊 **Progress Indicators**: See encryption/decryption progress in real-time
- 🎯 **Beginner-Friendly**: Simple CLI interface with clear instructions

### Advanced Features (Planned)
- 🧩 **Compression**: Automatic compression before encryption to save space
- 🔍 **File Integrity**: Built-in integrity checks to detect tampering
- 🚫 **Secure Deletion**: Option to securely delete original files after encryption
- 📝 **Batch Operations**: Encrypt multiple files/folders at once
- 🎨 **GUI Version**: Optional graphical interface for non-technical users

## 🛠️ Technical Approach

### Encryption Stack
- **Algorithm**: AES-256 in GCM mode (Galois/Counter Mode)
  - Provides both confidentiality and authenticity
  - Resistant to padding oracle attacks
  - NIST-approved standard

- **Key Derivation**: Argon2id
  - Memory-hard function resistant to GPU/ASIC attacks
  - Winner of the Password Hashing Competition
  - Configurable parameters for security/performance balance

- **Random Generation**: Cryptographically secure random number generator (CSPRNG)
  - For IVs, salts, and nonces
  - Platform-specific secure implementations

### File Format
```
[🏷️ Header]
- Magic bytes: "CRAT" (4 bytes)
- Version: 1 byte
- Algorithm ID: 1 byte
- Salt: 32 bytes (for key derivation)
- Nonce/IV: 12 bytes (for GCM)
- Metadata length: 4 bytes

[📝 Encrypted Metadata]
- Original filename (encrypted)
- Original file size
- Timestamp
- Compression flag

[🔒 Encrypted Data]
- The actual file contents (encrypted + compressed if enabled)

[✔️ Authentication Tag]
- GCM authentication tag: 16 bytes
```

## 🏭 Architecture

### Project Structure
```
cryptocrate/
├── src/
│   ├── main.rs              # Entry point and CLI
│   ├── crypto/
│   │   ├── mod.rs          # Crypto module
│   │   ├── encryption.rs   # Encryption/decryption logic
│   │   ├── key_derivation.rs # Argon2 key derivation
│   │   └── random.rs       # Secure random generation
│   ├── file_handler/
│   │   ├── mod.rs          # File handling module
│   │   ├── reader.rs       # File reading with streaming
│   │   ├── writer.rs       # File writing with streaming
│   │   └── walker.rs       # Directory traversal
│   ├── compression/
│   │   └── mod.rs          # Compression utilities
│   └── ui/
│       ├── mod.rs          # UI module
│       └── progress.rs     # Progress bars
├── tests/
│   ├── integration_tests.rs
│   └── crypto_tests.rs
├── benches/
│   └── encryption_bench.rs
├── Cargo.toml
├── README.md
└── LICENSE
```

### Technology Choice: Rust 🦀
**Why Rust?**
- Memory safety without garbage collection = fast and secure
- Excellent cryptography libraries (ring, RustCrypto)
- Cross-platform compilation (Windows, macOS, Linux)
- Great CLI tools ecosystem (clap, indicatif)
- Zero-cost abstractions for performance

### Key Dependencies
```toml
[dependencies]
aes-gcm = "0.10"         # AES-256-GCM encryption
argon2 = "0.5"          # Key derivation
rand = "0.8"            # Secure random generation
clap = "4.0"            # Command-line parsing
indicatif = "0.17"      # Progress bars
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

# Install globally (optional)
cargo install --path .
```

### Usage Examples

```bash
# Encrypt a single file
cryptocrate encrypt secret.txt

# Encrypt a folder
cryptocrate encrypt my_folder/

# Decrypt a file
cryptocrate decrypt secret.txt.crat

# Encrypt with compression
cryptocrate encrypt --compress large_file.dat

# Batch encrypt multiple items
cryptocrate encrypt file1.txt file2.txt folder1/ folder2/

# Decrypt to specific location
cryptocrate decrypt secret.txt.crat --output /path/to/output
```

## 🛣️ Roadmap

### Phase 1: Core Implementation (v0.1)
- [x] Project setup
- [ ] Basic AES-256-GCM encryption/decryption
- [ ] Argon2 key derivation
- [ ] Single file encryption
- [ ] Basic CLI interface
- [ ] Unit tests

### Phase 2: Enhanced Features (v0.2)
- [ ] Folder encryption with recursive traversal
- [ ] Progress bars for large files
- [ ] File metadata preservation
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
- **Unique salts and nonces**: Every encryption uses fresh random values
- **Memory safety**: Rust's ownership system prevents memory leaks and buffer overflows
- **No custom crypto**: Only battle-tested, peer-reviewed algorithms
- **Constant-time operations**: Where possible, to prevent timing attacks

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
- Use strong, unique passwords
- Keep your software updated
- Don't lose your passwords (we can't recover them!)

## 💬 Contact

Questions? Issues? Ideas? Open an issue or start a discussion!

---

Made with ❤️ and Rust 🦀