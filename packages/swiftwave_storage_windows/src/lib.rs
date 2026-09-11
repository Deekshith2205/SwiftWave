#![allow(unsafe_code)]

use std::path::{Path, PathBuf};
use std::ptr;
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
};
use windows_sys::Win32::Foundation::LocalFree;
use swiftwave_core::device::storage::SecureStorage;
use swiftwave_core::error::{Result, SwiftWaveError};

/// Secure storage implementation for Windows using DPAPI (Data Protection API).
/// 
/// DPAPI encrypts data using the user's logon credentials. The resulting opaque blob
/// is stored on disk and can only be decrypted by a process running as the same user.
pub struct WindowsSecureStorage {
    app_data_dir: PathBuf,
}

impl WindowsSecureStorage {
    /// Creates a new Windows secure storage instance.
    /// 
    /// The `app_data_dir` must be a directory isolated to the application,
    /// typically derived from `%APPDATA%` / `%LOCALAPPDATA%`.
    pub fn new(app_data_dir: impl AsRef<Path>) -> Result<Self> {
        let path = app_data_dir.as_ref().to_path_buf();
        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(SwiftWaveError::Io)?;
        }
        Ok(Self { app_data_dir: path })
    }

    fn key_to_filename(key: &str) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(key.len() * 2);
        for b in key.bytes() {
            write!(&mut s, "{:02x}", b).unwrap();
        }
        s
    }

    fn get_file_path(&self, key: &str) -> PathBuf {
        self.app_data_dir.join(Self::key_to_filename(key))
    }
}

impl SecureStorage for WindowsSecureStorage {
    fn save_secret(&self, key: &str, secret: &[u8]) -> Result<()> {
        let path = self.get_file_path(key);
        
        let mut data_in = CRYPT_INTEGER_BLOB {
            cbData: secret.len() as u32,
            pbData: secret.as_ptr() as *mut u8,
        };
        
        let mut data_out = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        // Call CryptProtectData to encrypt the byte array.
        let success = unsafe {
            CryptProtectData(
                &mut data_in,
                ptr::null(), // No description
                ptr::null(), // No optional entropy
                ptr::null_mut(), // No reserved
                ptr::null_mut(), // No prompt
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut data_out,
            )
        };

        if success == 0 {
            let err = std::io::Error::last_os_error();
            return Err(SwiftWaveError::Internal(format!("CryptProtectData failed: {}", err)));
        }

        if data_out.pbData.is_null() {
            return Err(SwiftWaveError::Internal("CryptProtectData returned null pointer".to_string()));
        }

        // Safely extract the encrypted bytes into a Vec
        let encrypted_blob = unsafe {
            std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
        };

        // Free the Windows-allocated memory
        unsafe {
            LocalFree(data_out.pbData as _);
        }

        // Atomically write the blob to disk.
        // We write to a .tmp file first (using process/thread ID to prevent races), then rename it.
        let temp_filename = format!(
            "{}.tmp.{}.{:?}", 
            Self::key_to_filename(key), 
            std::process::id(), 
            std::thread::current().id()
        );
        let temp_path = self.app_data_dir.join(temp_filename);
        
        if let Err(e) = std::fs::write(&temp_path, encrypted_blob) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(SwiftWaveError::Io(e));
        }

        if let Err(e) = std::fs::rename(&temp_path, &path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(SwiftWaveError::Io(e));
        }

        Ok(())
    }

    fn load_secret(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.get_file_path(key);
        
        let encrypted_blob = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(SwiftWaveError::Io(e)),
        };

        let mut data_in = CRYPT_INTEGER_BLOB {
            cbData: encrypted_blob.len() as u32,
            pbData: encrypted_blob.as_ptr() as *mut u8,
        };
        
        let mut data_out = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: ptr::null_mut(),
        };

        // Call CryptUnprotectData to decrypt the blob.
        let success = unsafe {
            CryptUnprotectData(
                &mut data_in,
                ptr::null_mut(), // Do not need description out
                ptr::null(),     // No optional entropy
                ptr::null_mut(), // No reserved
                ptr::null_mut(), // No prompt
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut data_out,
            )
        };

        if success == 0 {
            let err = std::io::Error::last_os_error();
            return Err(SwiftWaveError::Internal(format!("CryptUnprotectData failed: {}", err)));
        }

        if data_out.pbData.is_null() {
            return Err(SwiftWaveError::Internal("CryptUnprotectData returned null pointer".to_string()));
        }

        // Safely extract the decrypted bytes into a Vec
        let decrypted = unsafe {
            std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec()
        };

        // Free the Windows-allocated memory immediately
        unsafe {
            LocalFree(data_out.pbData as _);
        }

        Ok(Some(decrypted))
    }

    fn delete_secret(&self, key: &str) -> Result<()> {
        let path = self.get_file_path(key);
        match std::fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => Err(SwiftWaveError::Io(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_dpapi_first_load_returns_none() {
        let dir = tempdir().unwrap();
        let storage = WindowsSecureStorage::new(dir.path()).unwrap();
        let result = storage.load_secret("test_key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_dpapi_save_load_round_trip() {
        let dir = tempdir().unwrap();
        let storage = WindowsSecureStorage::new(dir.path()).unwrap();
        
        let secret = b"my arbitrary 32 byte secret data";
        assert_eq!(secret.len(), 32);

        storage.save_secret("identity_key", secret).unwrap();
        
        let loaded = storage.load_secret("identity_key").unwrap().unwrap();
        assert_eq!(loaded, secret);

        // Security check: Verify the file actually exists and its contents are NOT plaintext.
        let file_path = storage.get_file_path("identity_key");
        assert!(file_path.exists());
        let disk_bytes = std::fs::read(&file_path).unwrap();
        assert_ne!(disk_bytes, secret); // Ensure DPAPI encrypted it
    }

    #[test]
    fn test_dpapi_overwrite() {
        let dir = tempdir().unwrap();
        let storage = WindowsSecureStorage::new(dir.path()).unwrap();
        
        storage.save_secret("key1", b"first_data").unwrap();
        storage.save_secret("key1", b"second_data").unwrap();
        
        let loaded = storage.load_secret("key1").unwrap().unwrap();
        assert_eq!(loaded, b"second_data");
    }

    #[test]
    fn test_dpapi_delete() {
        let dir = tempdir().unwrap();
        let storage = WindowsSecureStorage::new(dir.path()).unwrap();
        
        storage.save_secret("key_to_delete", b"delete_me").unwrap();
        assert!(storage.load_secret("key_to_delete").unwrap().is_some());
        
        storage.delete_secret("key_to_delete").unwrap();
        assert!(storage.load_secret("key_to_delete").unwrap().is_none());
        
        // Deleting again should not fail
        assert!(storage.delete_secret("key_to_delete").is_ok());
    }

    #[test]
    fn test_dpapi_corrupted_blob_returns_error() {
        let dir = tempdir().unwrap();
        let storage = WindowsSecureStorage::new(dir.path()).unwrap();
        
        let file_path = storage.get_file_path("corrupt_key");
        std::fs::write(&file_path, b"not a valid dpapi blob").unwrap();
        
        let result = storage.load_secret("corrupt_key");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SwiftWaveError::Internal(_)));
    }

    #[test]
    fn test_dpapi_persistence_across_instances() {
        let dir = tempdir().unwrap();
        
        let storage1 = WindowsSecureStorage::new(dir.path()).unwrap();
        storage1.save_secret("persistent_key", b"persisted_data").unwrap();
        
        // Second instance pointing to the exact same directory (simulating restart)
        let storage2 = WindowsSecureStorage::new(dir.path()).unwrap();
        let loaded = storage2.load_secret("persistent_key").unwrap().unwrap();
        
        assert_eq!(loaded, b"persisted_data");
    }

    #[test]
    fn test_dpapi_concurrent_saves() {
        let dir = tempdir().unwrap();
        let storage = Arc::new(WindowsSecureStorage::new(dir.path()).unwrap());
        
        let mut handles = vec![];
        for i in 0..10 {
            let s = storage.clone();
            handles.push(thread::spawn(move || {
                let data = format!("data_{}", i).into_bytes();
                s.save_secret("concurrent_key", &data).unwrap();
            }));
        }
        
        for h in handles {
            h.join().unwrap();
        }
        
        // Final state should be perfectly readable without DPAPI corruption
        let loaded = storage.load_secret("concurrent_key").unwrap().unwrap();
        let s = String::from_utf8(loaded).unwrap();
        assert!(s.starts_with("data_")); // One of the threads won the race cleanly.
    }
}
