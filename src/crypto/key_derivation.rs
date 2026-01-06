//! Key derivation using Argon2id

use argon2::{Argon2, ParamsBuilder, Version};
use crate::error::{CrateError, Result};
use crate::format::KEY_LENGTH;

/// Derive a cryptographic key from a password using Argon2id
///
/// # Parameters
/// - `password`: The user's password
/// - `salt`: A unique salt for this encryption operation
///
/// # Security
/// Uses Argon2id with recommended parameters:
/// - Memory cost: 64 MB
/// - Time cost: 3 iterations
/// - Parallelism: 4 threads
pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LENGTH]> {
    // Configure Argon2id parameters
    let mut params_builder = ParamsBuilder::new();
    params_builder
        .m_cost(65536) // 64 MB
        .t_cost(3) // 3 iterations
        .p_cost(4) // 4 parallel threads
        .output_len(KEY_LENGTH)
        .map_err(|e| CrateError::KeyDerivation(e.to_string()))?;

    let params = params_builder
        .build()
        .map_err(|e| CrateError::KeyDerivation(e.to_string()))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        Version::V0x13,
        params,
    );

    let mut key = [0u8; KEY_LENGTH];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| CrateError::KeyDerivation(e.to_string()))?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation() {
        let password = "test_password";
        let salt = [0u8; 32];

        let key = derive_key(password, &salt).unwrap();
        assert_eq!(key.len(), KEY_LENGTH);

        // Same password and salt should produce same key
        let key2 = derive_key(password, &salt).unwrap();
        assert_eq!(key, key2);

        // Different salt should produce different key
        let salt2 = [1u8; 32];
        let key3 = derive_key(password, &salt2).unwrap();
        assert_ne!(key, key3);
    }
}
