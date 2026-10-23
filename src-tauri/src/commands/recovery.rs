use crate::{
    types::{MAX_RECOVERY_DATA_BYTES, ServiceError, validate_filename},
    utils::storage,
};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use tauri::State;

pub struct RecoveryStore {
    root: PathBuf,
    lock: Mutex<()>,
}

impl RecoveryStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            lock: Mutex::new(()),
        }
    }
    pub fn save(&self, filename: &str, data: &Value) -> Result<(), ServiceError> {
        validate_filename(filename)?;
        let _guard = self.lock.lock().map_err(|_| ServiceError::Unavailable)?;
        storage::write_json(
            &self.root.join(format!("{filename}.json")),
            data,
            MAX_RECOVERY_DATA_BYTES,
        )
    }
    pub fn load(&self, filename: &str) -> Result<Value, ServiceError> {
        validate_filename(filename)?;
        let _guard = self.lock.lock().map_err(|_| ServiceError::Unavailable)?;
        storage::read_json(
            &self.root.join(format!("{filename}.json")),
            MAX_RECOVERY_DATA_BYTES,
        )
    }
    pub fn cleanup(&self) -> Result<u32, ServiceError> {
        let _guard = self.lock.lock().map_err(|_| ServiceError::Unavailable)?;
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(_) => return Err(ServiceError::Io),
        };
        let cutoff = SystemTime::now() - Duration::from_secs(7 * 86_400);
        let mut removed = 0;
        for entry in entries {
            let entry = entry.map_err(|_| ServiceError::Io)?;
            let kind = entry.file_type().map_err(|_| ServiceError::Io)?;
            if !kind.is_file()
                || entry
                    .path()
                    .extension()
                    .is_none_or(|extension| extension != "json")
            {
                continue;
            }
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .map_err(|_| ServiceError::Io)?;
            if modified < cutoff {
                fs::remove_file(entry.path()).map_err(|_| ServiceError::Io)?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}

#[tauri::command]
#[specta::specta]
pub async fn save_emergency_data(
    store: State<'_, Arc<RecoveryStore>>,
    filename: String,
    data: Value,
) -> Result<(), ServiceError> {
    let store = store.inner().clone();
    storage::blocking(move || store.save(&filename, &data)).await
}
#[tauri::command]
#[specta::specta]
pub async fn load_emergency_data(
    store: State<'_, Arc<RecoveryStore>>,
    filename: String,
) -> Result<Value, ServiceError> {
    let store = store.inner().clone();
    storage::blocking(move || store.load(&filename)).await
}
#[tauri::command]
#[specta::specta]
pub async fn cleanup_old_recovery_files(
    store: State<'_, Arc<RecoveryStore>>,
) -> Result<u32, ServiceError> {
    let store = store.inner().clone();
    storage::blocking(move || store.cleanup()).await
}
