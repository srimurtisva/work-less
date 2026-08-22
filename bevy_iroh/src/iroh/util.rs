use iroh::SecretKey;
use directories::ProjectDirs;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

pub struct NodeIdentityManager {
    key_path: PathBuf,
}

impl NodeIdentityManager {
    /// Создает менеджер для тестовых или кастомных путей.
    pub fn new<P: AsRef<Path>>(data_dir: P) -> Self {
        Self {
            key_path: data_dir.as_ref().join("identity.secret"),
        }
    }

    /// Автоматически определяет пути на основе имени запущенного файла.
    /// Не пробрасывает ошибки: при сбоях логирует их и использует локальный фолбэк.
    pub fn auto() -> Self {
        let app_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| {
                warn!("Не удалось определить имя бинарника, используем 'unknown_app'");
                "unknown_app".to_string()
            });

        let data_dir = ProjectDirs::from("", "", &app_name)
            .map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                // Если нет домашней директории (например, в Docker без $HOME), 
                // падаем в локальную папку
                let fallback = PathBuf::from("./.data").join(&app_name);
                warn!("Системные директории недоступны. Фолбэк на {:?}", fallback);
                fallback
            });

        if let Err(e) = fs::create_dir_all(&data_dir) {
            error!("Ошибка создания директории {:?}: {}", data_dir, e);
        }

        Self::new(data_dir)
    }

    /// Возвращает существующий ключ или создает новый. 
    /// Все ошибки I/O логируются, приложение не падает.
    pub fn get_or_create_key(&self) -> SecretKey {
        if self.key_path.exists() {
            match self.load_key() {
                Some(key) => {
                    info!("Node ID успешно загружен: {}", key.public());
                    return key;
                }
                None => warn!("Файл ключа поврежден или недоступен. Генерируем новый..."),
            }
        }

        let new_key = SecretKey::generate();
        self.save_key(&new_key);
        
        info!("Сгенерирован новый Node ID: {}", new_key.public());
        new_key
    }

    // --- Внутренние методы (скрыты от публичного API) ---

    fn load_key(&self) -> Option<SecretKey> {
        let mut file = fs::File::open(&self.key_path).map_err(|e| {
            error!("Не удалось открыть файл ключа {:?}: {}", self.key_path, e);
        }).ok()?;

        let mut key_bytes = [0u8; 32];
        file.read_exact(&mut key_bytes).map_err(|e| {
            error!("Ошибка чтения байт ключа: {}", e);
        }).ok()?;

        Some(SecretKey::from_bytes(&key_bytes))
    }

    fn save_key(&self, key: &SecretKey) {
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        match options.open(&self.key_path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(&key.to_bytes()) {
                    error!("Не удалось записать ключ на диск: {}", e);
                }
            }
            Err(e) => {
                error!("Не удалось создать файл ключа {:?}: {}", self.key_path, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // В тестах не инициализируем tracing-subscriber, 
    // поэтому логи просто уйдут в никуда, не мешая проверкам.

    #[test]
    fn test_generates_new_key_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let manager = NodeIdentityManager::new(temp_dir.path());

        let key1 = manager.get_or_create_key();
        assert!(manager.key_path.exists());
        
        let key2 = manager.get_or_create_key();
        assert_eq!(key1.public(), key2.public());
    }

    #[test]
    fn test_corrupted_key_handling() {
        let temp_dir = TempDir::new().unwrap();
        let manager = NodeIdentityManager::new(temp_dir.path());

        // Симулируем повреждение: пишем мусор
        fs::write(&manager.key_path, b"too_short").unwrap();

        // Метод должен залогировать ошибку, проигнорировать мусор и сгенерировать новый ключ
        let key = manager.get_or_create_key();
        
        // Убеждаемся, что ключ валидный (вызов public() не паникует)
        let _pub = key.public();
    }

    #[test]
    #[cfg(unix)]
    fn test_file_permissions_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let manager = NodeIdentityManager::new(temp_dir.path());

        manager.get_or_create_key();

        let metadata = fs::metadata(&manager.key_path).unwrap();
        let permissions = metadata.permissions();
        assert_eq!(permissions.mode() & 0o777, 0o600);
    }
    
    #[test]
    fn test_auto_does_not_panic() {
        let manager = NodeIdentityManager::auto();
        // Просто проверяем, что структура создалась
        assert!(manager.key_path.parent().is_some());
    }
}