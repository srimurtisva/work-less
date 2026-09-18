use directories::ProjectDirs;
use fs2::FileExt;
use iroh::SecretKey;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

pub struct NodeIdentityManager {
    key_path: PathBuf,
    lock_path: PathBuf,

    // The file handle must live for as long as the identity is in use.
    // Dropping it releases the OS-level file lock.
    lock_file: Option<File>,
}

pub struct NodeIdentity {
    pub key: SecretKey,

    /// True if this instance owns the persistent device identity.
    ///
    /// False if another instance is already using the persistent
    /// identity and this instance therefore uses a new ephemeral key.
    pub owns_persistent_identity: bool,
}

impl NodeIdentityManager {
    /// Creates a manager for a custom data directory.
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        let data_dir = data_dir.as_ref();

        Self {
            key_path: data_dir.join("identity.secret"),
            lock_path: data_dir.join("identity.lock"),
            lock_file: None,
        }
    }

    /// Automatically determines the application data directory.
    pub fn auto() -> Self {
        let app_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| {
                warn!("Could not determine executable name; using 'unknown_app'");
                "unknown_app".to_string()
            });

        let data_dir = ProjectDirs::from("", "", &app_name)
            .map(|project_dirs| project_dirs.data_local_dir().to_path_buf())
            .unwrap_or_else(|| {
                // Fall back to a local directory when system directories
                // are unavailable, for example in a container without HOME.
                let fallback = PathBuf::from("./.data").join(&app_name);
                warn!(
                    "System data directories are unavailable; \
                     falling back to {:?}",
                    fallback
                );
                fallback
            });

        if let Err(e) = fs::create_dir_all(&data_dir) {
            error!(
                "Failed to create data directory {:?}: {}",
                data_dir, e
            );
        }

        Self::new(data_dir)
    }

    /// Acquires the identity for this application instance.
    ///
    /// If the persistent identity lock can be acquired, the stored
    /// persistent key is loaded or created.
    ///
    /// If another process already owns the lock, a new key is generated
    /// for this instance without modifying the persistent key.
    ///
    /// If acquiring the lock fails for any other reason, a new key is
    /// generated as a safe fallback.
    ///
    /// This method is intended to be called once during application startup.
    pub fn acquire_identity(&mut self) -> NodeIdentity {
        match self.try_acquire_lock() {
            Ok(true) => {
                let key = self.get_or_create_persistent_key();

                info!("Using persistent Node ID: {}", key.public());

                NodeIdentity {
                    key,
                    owns_persistent_identity: true,
                }
            }

            Ok(false) => {
                warn!(
                    "Another application instance already owns the \
                     persistent Node ID; generating a separate Node ID"
                );

                let key = SecretKey::generate();

                info!("Generated ephemeral Node ID: {}", key.public());

                NodeIdentity {
                    key,
                    owns_persistent_identity: false,
                }
            }

            Err(e) => {
                // We cannot safely determine whether another instance
                // owns the persistent identity, so do not use it.
                error!(
                    "Failed to acquire identity lock {:?}: {}",
                    self.lock_path, e
                );

                let key = SecretKey::generate();

                warn!(
                    "Using a new Node ID because the identity lock \
                     could not be acquired: {}",
                    key.public()
                );

                NodeIdentity {
                    key,
                    owns_persistent_identity: false,
                }
            }
        }
    }

    /// Attempts to acquire the exclusive identity lock.
    ///
    /// Returns:
    /// - Ok(true) if the lock was acquired;
    /// - Ok(false) if another process already owns the lock;
    /// - Err(...) for an actual I/O or OS error.
    fn try_acquire_lock(&mut self) -> std::io::Result<bool> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&self.lock_path)?;

        match file.try_lock_exclusive() {
            Ok(()) => {
                // Keep the file handle alive because dropping it releases
                // the OS-level lock.
                self.lock_file = Some(file);
                Ok(true)
            }

            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Another process owns the lock.
                Ok(false)
            }

            Err(e) => Err(e),
        }
    }

    fn get_or_create_persistent_key(&self) -> SecretKey {
        if self.key_path.exists() {
            match self.load_key() {
                Some(key) => {
                    info!("Loaded persistent Node ID: {}", key.public());
                    return key;
                }

                None => {
                    warn!(
                        "Identity file is corrupted or inaccessible; \
                         generating a new persistent key"
                    );
                }
            }
        }

        let new_key = SecretKey::generate();
        self.save_key(&new_key);

        info!(
            "Generated new persistent Node ID: {}",
            new_key.public()
        );

        new_key
    }

    fn load_key(&self) -> Option<SecretKey> {
        let mut file = fs::File::open(&self.key_path)
            .map_err(|e| {
                error!(
                    "Failed to open identity file {:?}: {}",
                    self.key_path, e
                );
            })
            .ok()?;

        let mut key_bytes = [0u8; 32];

        file.read_exact(&mut key_bytes)
            .map_err(|e| {
                error!("Failed to read identity key: {}", e);
            })
            .ok()?;

        Some(SecretKey::from_bytes(&key_bytes))
    }

    fn save_key(&self, key: &SecretKey) {
        let mut options = OpenOptions::new();

        options
            .write(true)
            .create(true)
            .truncate(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        match options.open(&self.key_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&key.to_bytes()) {
                    error!(
                        "Failed to write identity key to disk: {}",
                        e
                    );
                }
            }

            Err(e) => {
                error!(
                    "Failed to create identity file {:?}: {}",
                    self.key_path, e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generates_and_persists_key() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = NodeIdentityManager::new(temp_dir.path());

        let identity = manager.acquire_identity();

        assert!(identity.owns_persistent_identity);
        assert!(manager.key_path.exists());
        assert!(manager.lock_path.exists());
    }

    #[test]
    fn test_persistent_key_survives_manager_restart() {
        let temp_dir = TempDir::new().unwrap();

        let persistent_id = {
            let mut manager = NodeIdentityManager::new(temp_dir.path());
            let identity = manager.acquire_identity();

            assert!(identity.owns_persistent_identity);

            identity.key.public()
        };

        // The first manager has been dropped, so its lock is released.
        let mut manager = NodeIdentityManager::new(temp_dir.path());
        let identity = manager.acquire_identity();

        assert!(identity.owns_persistent_identity);
        assert_eq!(persistent_id, identity.key.public());
    }

    #[test]
    fn test_second_instance_gets_ephemeral_key() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager1 = NodeIdentityManager::new(temp_dir.path());
        let identity1 = manager1.acquire_identity();

        assert!(identity1.owns_persistent_identity);

        let mut manager2 = NodeIdentityManager::new(temp_dir.path());
        let identity2 = manager2.acquire_identity();

        assert!(!identity2.owns_persistent_identity);

        // The second instance must not reuse the persistent identity.
        assert_ne!(
            identity1.key.public(),
            identity2.key.public()
        );
    }

    #[test]
    fn test_second_instance_does_not_modify_persistent_key() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager1 = NodeIdentityManager::new(temp_dir.path());
        let identity1 = manager1.acquire_identity();

        let persistent_id = identity1.key.public();

        let mut manager2 = NodeIdentityManager::new(temp_dir.path());
        let identity2 = manager2.acquire_identity();

        assert!(!identity2.owns_persistent_identity);

        // The second instance must not overwrite the persistent key.
        let stored_key = fs::read(&manager1.key_path).unwrap();

        assert_eq!(stored_key.len(), 32);

        let stored_key: [u8; 32] = stored_key.try_into().unwrap();
        let stored_key = SecretKey::from_bytes(&stored_key);

        assert_eq!(persistent_id, stored_key.public());
    }

    #[test]
    fn test_lock_is_released_when_manager_is_dropped() {
        let temp_dir = TempDir::new().unwrap();

        let persistent_id = {
            let mut manager1 = NodeIdentityManager::new(temp_dir.path());
            let identity1 = manager1.acquire_identity();

            assert!(identity1.owns_persistent_identity);

            identity1.key.public()
        };

        // manager1 was dropped, so the OS-level lock was released.
        let mut manager2 = NodeIdentityManager::new(temp_dir.path());
        let identity2 = manager2.acquire_identity();

        assert!(identity2.owns_persistent_identity);
        assert_eq!(persistent_id, identity2.key.public());
    }

    #[test]
    fn test_corrupted_key_handling() {
        let temp_dir = TempDir::new().unwrap();

        let mut manager = NodeIdentityManager::new(temp_dir.path());

        fs::write(&manager.key_path, b"too_short").unwrap();

        let identity = manager.acquire_identity();

        assert!(identity.owns_persistent_identity);

        let persisted_key = fs::read(&manager.key_path).unwrap();

        assert_eq!(persisted_key.len(), 32);

        let persisted_key: [u8; 32] =
            persisted_key.try_into().unwrap();

        let persisted_key = SecretKey::from_bytes(&persisted_key);

        assert_eq!(
            identity.key.public(),
            persisted_key.public()
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_file_permissions_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let mut manager = NodeIdentityManager::new(temp_dir.path());

        manager.acquire_identity();

        let metadata = fs::metadata(&manager.key_path).unwrap();
        let permissions = metadata.permissions();

        assert_eq!(permissions.mode() & 0o777, 0o600);
    }

    #[test]
    fn test_auto_does_not_panic() {
        let manager = NodeIdentityManager::auto();

        assert!(manager.key_path.parent().is_some());
        assert!(manager.lock_path.parent().is_some());
    }
}
 
