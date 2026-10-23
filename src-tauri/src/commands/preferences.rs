use crate::{
    types::{AppPreferences, ServiceError, Theme},
    utils::storage,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::State;

pub struct PreferencesStore {
    root: PathBuf,
    lock: Mutex<()>,
}

impl PreferencesStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            lock: Mutex::new(()),
        }
    }
    fn read(&self) -> Result<AppPreferences, ServiceError> {
        let result =
            storage::read_json::<AppPreferences>(&self.root.join("preferences.json"), 65_536);
        match result {
            Ok(value) => {
                value.validate().map_err(|_| ServiceError::CorruptData)?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }
    pub fn load(&self) -> Result<AppPreferences, ServiceError> {
        let _guard = self.lock.lock().map_err(|_| ServiceError::Unavailable)?;
        match self.read() {
            Err(ServiceError::NotFound) => Ok(AppPreferences::default()),
            result => result,
        }
    }
    pub fn save(&self, preferences: &AppPreferences) -> Result<(), ServiceError> {
        preferences.validate()?;
        let _guard = self.lock.lock().map_err(|_| ServiceError::Unavailable)?;
        storage::write_json(&self.root.join("preferences.json"), preferences, 65_536)
    }
    pub fn migrate_legacy_theme(&self, theme: Theme) -> Result<AppPreferences, ServiceError> {
        let _guard = self.lock.lock().map_err(|_| ServiceError::Unavailable)?;
        match self.read() {
            Err(ServiceError::NotFound) => {
                let preferences = AppPreferences {
                    theme,
                    ..AppPreferences::default()
                };
                storage::write_json(&self.root.join("preferences.json"), &preferences, 65_536)?;
                Ok(preferences)
            }
            result => result,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn load_preferences(
    store: State<'_, Arc<PreferencesStore>>,
) -> Result<AppPreferences, ServiceError> {
    let store = store.inner().clone();
    storage::blocking(move || store.load()).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_preferences(
    store: State<'_, Arc<PreferencesStore>>,
    preferences: AppPreferences,
) -> Result<(), ServiceError> {
    let store = store.inner().clone();
    storage::blocking(move || store.save(&preferences)).await
}

#[tauri::command]
#[specta::specta]
pub async fn migrate_legacy_theme(
    store: State<'_, Arc<PreferencesStore>>,
    theme: Theme,
) -> Result<AppPreferences, ServiceError> {
    let store = store.inner().clone();
    storage::blocking(move || store.migrate_legacy_theme(theme)).await
}
