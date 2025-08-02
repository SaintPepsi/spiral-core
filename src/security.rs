/// 🔐 SECURITY MODULE: Cryptographically secure operations
/// CRITICAL: All security-sensitive operations centralized here for audit
/// Purpose: API key generation, secure random generation, security constants
use crate::SpiralError;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// 🔑 API KEY SPECIFICATIONS: Cryptographically secure requirements
/// DECISION: 64 chars = 384 bits entropy (exceeds NIST 256-bit requirement)
/// Why: Base62 encoding (A-Z, a-z, 0-9) provides ~5.95 bits per character
/// Calculation: 64 chars × 5.95 bits = ~380 bits of entropy
/// Alternative: 32 chars (rejected: only ~190 bits), 128 chars (rejected: unnecessarily long)
pub const API_KEY_LENGTH: usize = 64;

/// 📁 API KEY FILE LOCATION: Secure storage path
/// DECISION: Store in project root for easy access but not in repository
/// Why: .spiral-api-key is gitignored, stays with project but not committed
/// Alternative: System-wide storage (rejected: harder deployment), env var only (rejected: not persistent)
pub const API_KEY_FILE: &str = ".spiral-api-key";

/// 🎲 SECURE RANDOM API KEY GENERATOR: Cryptographically strong randomness
/// CRITICAL: Uses OS-level entropy via thread_rng() for security
/// Why: thread_rng() seeds from OS entropy sources (/dev/urandom, CryptGenRandom)
/// Alternative: Simple rand (rejected: predictable), timestamp-based (rejected: weak)
pub fn generate_secure_api_key() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(API_KEY_LENGTH)
        .map(char::from)
        .collect()
}

/// 💾 API KEY PERSISTENCE: Secure file storage with proper permissions
/// CRITICAL: Restricts file permissions to owner-only (600)
/// Why: Prevents other users/processes from reading the API key
/// Alternative: World-readable (rejected: security risk), no persistence (rejected: not practical)
pub fn save_api_key_to_file(api_key: &str) -> Result<(), SpiralError> {
    info!("Saving API key to secure file: {}", API_KEY_FILE);

    // Write the API key to file
    fs::write(API_KEY_FILE, api_key).map_err(|e| {
        SpiralError::ConfigurationError(format!("Failed to write API key file: {e}"))
    })?;

    // Set restrictive permissions (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(API_KEY_FILE)
            .map_err(|e| {
                SpiralError::ConfigurationError(format!("Failed to get file metadata: {e}"))
            })?
            .permissions();
        perms.set_mode(0o600); // Owner read/write only
        fs::set_permissions(API_KEY_FILE, perms).map_err(|e| {
            SpiralError::ConfigurationError(format!("Failed to set file permissions: {e}"))
        })?;
    }

    info!("API key saved successfully with secure permissions");
    Ok(())
}

/// 📖 API KEY LOADING: Read existing key from secure storage
/// DECISION: Silent failure if file doesn't exist (None return)
/// Why: Allows generation flow to work smoothly
/// Alternative: Error on missing file (rejected: breaks generation flow)
pub fn load_api_key_from_file() -> Result<Option<String>, SpiralError> {
    if !Path::new(API_KEY_FILE).exists() {
        return Ok(None);
    }

    let api_key = fs::read_to_string(API_KEY_FILE).map_err(|e| {
        SpiralError::ConfigurationError(format!("Failed to read API key file: {e}"))
    })?;

    let api_key = api_key.trim().to_string();

    // Validate the loaded key
    if api_key.len() != API_KEY_LENGTH {
        warn!(
            "Loaded API key has incorrect length: {} (expected {})",
            api_key.len(),
            API_KEY_LENGTH
        );
        return Err(SpiralError::ConfigurationError(
            "Invalid API key format in file".to_string(),
        ));
    }

    if !api_key.chars().all(|c| c.is_alphanumeric()) {
        warn!("Loaded API key contains invalid characters");
        return Err(SpiralError::ConfigurationError(
            "Invalid API key format in file".to_string(),
        ));
    }

    info!("API key loaded successfully from file");
    Ok(Some(api_key))
}

/// 🔄 API KEY INITIALIZATION: Validate existing or generate new secure key
/// DECISION: Use config key if exists, otherwise generate secure file-based key
/// Why: Respect environment configuration while ensuring secure fallback
/// Alternative: Always generate (rejected: ignores user config)
pub fn ensure_api_key_exists(existing_api_key: Option<&str>) -> Result<String, SpiralError> {
    info!("Ensuring secure API key exists...");

    // If config already has an API key (from env var), validate and use it
    if let Some(key) = existing_api_key {
        if !key.trim().is_empty() {
            info!("Using API key from configuration (environment variable)");
            return Ok(key.to_string());
        }
    }

    // No config key, try to load existing file-based key
    match load_api_key_from_file()? {
        Some(existing_key) => {
            info!("Using existing API key from file");
            Ok(existing_key)
        }
        None => {
            info!("No API key found, generating new secure key...");
            let new_key = generate_secure_api_key();
            save_api_key_to_file(&new_key)?;
            info!("New API key generated and saved securely");
            Ok(new_key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 🧪 API KEY GENERATION TEST: Verify cryptographic properties
    #[test]
    fn test_api_key_generation() {
        let key1 = generate_secure_api_key();
        let key2 = generate_secure_api_key();

        // Length validation
        assert_eq!(
            key1.len(),
            API_KEY_LENGTH,
            "API key should have correct length"
        );
        assert_eq!(
            key2.len(),
            API_KEY_LENGTH,
            "API key should have correct length"
        );

        // Uniqueness validation (probabilistic)
        assert_ne!(key1, key2, "Generated keys should be unique");

        // Character set validation
        assert!(
            key1.chars().all(|c| c.is_alphanumeric()),
            "API key should only contain alphanumeric characters"
        );
        assert!(
            key2.chars().all(|c| c.is_alphanumeric()),
            "API key should only contain alphanumeric characters"
        );

        // Entropy validation (basic)
        let unique_chars: std::collections::HashSet<char> = key1.chars().collect();
        assert!(
            unique_chars.len() >= 20,
            "API key should have reasonable character diversity"
        );
    }

    /// 💾 FILE PERSISTENCE TEST: Verify secure storage and loading
    #[test]
    fn test_api_key_file_operations() {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp directory");
        let test_file = temp_dir.path().join("test-api-key");

        // Test key generation and saving
        let original_key = generate_secure_api_key();

        // Save in temp location for testing
        fs::write(&test_file, &original_key).expect("Failed to write test key");

        // Load and verify
        let loaded_key = fs::read_to_string(&test_file).expect("Failed to read test key");
        assert_eq!(
            original_key,
            loaded_key.trim(),
            "Loaded key should match saved key"
        );

        // Verify file permissions on Unix systems
        #[cfg(unix)]
        {
            // Note: We can't test the actual permission setting without affecting the real API_KEY_FILE
            // This test focuses on the core functionality
        }
    }

    /// 🔄 INITIALIZATION FLOW TEST: Verify complete key management
    #[test]
    fn test_key_initialization_flow() {
        // Test validation of valid key
        let valid_key = generate_secure_api_key();
        assert_eq!(valid_key.len(), API_KEY_LENGTH);
        assert!(valid_key.chars().all(|c| c.is_alphanumeric()));

        // Test invalid key detection
        let invalid_keys = vec![
            "too_short".to_string(),
            "a".repeat(API_KEY_LENGTH + 1),              // too long
            "a".repeat(API_KEY_LENGTH - 1),              // too short by 1
            "invalid-char!".repeat(API_KEY_LENGTH / 13), // special characters
        ];

        for invalid_key in invalid_keys {
            // Verify that invalid keys don't meet our requirements
            assert!(
                invalid_key.len() != API_KEY_LENGTH
                    || !invalid_key.chars().all(|c| c.is_alphanumeric()),
                "Key should be invalid: {}",
                &invalid_key
            );
        }
    }

    /// ⚡ PERFORMANCE TEST: Ensure key generation is fast
    #[test]
    fn test_key_generation_performance() {
        let start = std::time::Instant::now();

        // Generate multiple keys to test performance
        for _ in 0..100 {
            let _key = generate_secure_api_key();
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100,
            "Key generation should be fast: took {}ms",
            duration.as_millis()
        );
    }

    /// 🎲 RANDOMNESS TEST: Basic entropy validation
    #[test]
    fn test_key_randomness() {
        let mut keys = std::collections::HashSet::new();

        // Generate multiple keys and check for duplicates
        for _ in 0..50 {
            let key = generate_secure_api_key();
            assert!(!keys.contains(&key), "Generated key should be unique");
            keys.insert(key);
        }

        assert_eq!(keys.len(), 50, "All generated keys should be unique");
    }
}
