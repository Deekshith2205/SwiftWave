//! Platform-agnostic secure storage abstraction.
//!
//! SwiftWave core delegates the actual persistence of sensitive identity material
//! (e.g., the X25519 static private key) to the host platform via this trait.
//! 
//! # Platform Implementations
//! - **Android**: Android Keystore system.
//! - **iOS/macOS**: Keychain Services.
//! - **Windows**: Data Protection API (DPAPI) or Credential Manager.
//! - **Linux**: Secret Service API or a fallback encrypted file.
//!
//! Using an abstraction ensures that raw secret material is never written
//! to plaintext files by the core engine.

use crate::error::{Result, SwiftWaveError};

/// A key used to identify a stored secret.
pub const IDENTITY_SECRET_KEY: &str = "swiftwave_device_identity";

/// An abstraction for platform-provided secure storage.
///
/// This trait is intended to be implemented by platform-specific adapters
/// (e.g., in Flutter via FFI calls down to native code) and injected into
/// the SwiftWave core at startup.
pub trait SecureStorage: Send + Sync {
    /// Save a secret byte array under the given key.
    fn save_secret(&self, key: &str, secret: &[u8]) -> Result<()>;

    /// Load a secret byte array by key.
    /// Returns `Ok(None)` if the key does not exist.
    fn load_secret(&self, key: &str) -> Result<Option<Vec<u8>>>;
    
    /// Delete a secret by key.
    fn delete_secret(&self, key: &str) -> Result<()>;
}

/// A fallback storage implementation that saves secrets to a local file.
///
/// # Security note
/// This should **only** be used for testing or on platforms without a
/// hardware-backed or OS-level secure keystore. It stores secrets in
/// plaintext JSON!
#[cfg(feature = "fallback_storage")]
pub mod fallback {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// File-based fallback storage.
    pub struct FileSecureStorage {
        path: PathBuf,
        cache: Mutex<HashMap<String, Vec<u8>>>,
    }

    impl FileSecureStorage {
        /// Create a new fallback storage backed by the given file path.
        pub fn new(path: impl AsRef<Path>) -> Result<Self> {
            let path = path.as_ref().to_path_buf();
            let mut cache = HashMap::new();
            
            if path.exists() {
                let data = std::fs::read_to_string(&path)
                    .map_err(SwiftWaveError::Io)?;
                let hex_map: HashMap<String, String> = serde_json::from_str(&data)
                    .map_err(SwiftWaveError::Serialisation)?;
                
                for (k, v) in hex_map {
                    if let Ok(bytes) = hex::decode(&v) {
                        cache.insert(k, bytes);
                    }
                }
            }

            Ok(Self {
                path,
                cache: Mutex::new(cache),
            })
        }

        fn persist(&self, cache: &HashMap<String, Vec<u8>>) -> Result<()> {
            if let Some(parent) = self.path.parent() {
                std::fs::create_dir_all(parent).map_err(SwiftWaveError::Io)?;
            }
            
            let mut hex_map = HashMap::new();
            for (k, v) in cache {
                hex_map.insert(k.clone(), hex::encode(v));
            }
            
            let json = serde_json::to_string_pretty(&hex_map)
                .map_err(SwiftWaveError::Serialisation)?;
            std::fs::write(&self.path, json).map_err(SwiftWaveError::Io)?;
            
            // Attempt to restrict file permissions on Unix.
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mut perms) = std::fs::metadata(&self.path).map(|m| m.permissions()) {
                    perms.set_mode(0o600);
                    let _ = std::fs::set_permissions(&self.path, perms);
                }
            }
            
            Ok(())
        }
    }

    impl SecureStorage for FileSecureStorage {
        fn save_secret(&self, key: &str, secret: &[u8]) -> Result<()> {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key.to_string(), secret.to_vec());
            self.persist(&cache)
        }

        fn load_secret(&self, key: &str) -> Result<Option<Vec<u8>>> {
            let cache = self.cache.lock().unwrap();
            Ok(cache.get(key).cloned())
        }

        fn delete_secret(&self, key: &str) -> Result<()> {
            let mut cache = self.cache.lock().unwrap();
            if cache.remove(key).is_some() {
                self.persist(&cache)?;
            }
            Ok(())
        }
    }
}
