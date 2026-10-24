use super::{effects::ProductError, settings::ProductSettings};
use std::{path::PathBuf, sync::Mutex};

pub struct SettingsStore {
    root: PathBuf,
    lock: Mutex<()>,
}
impl SettingsStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            lock: Mutex::new(()),
        }
    }
    pub fn load(&self) -> Result<ProductSettings, ProductError> {
        let _guard = self.lock.lock().map_err(|_| ProductError::Unavailable)?;
        self.read()
    }
    fn read(&self) -> Result<ProductSettings, ProductError> {
        match crate::utils::storage::read_json::<ProductSettings>(
            &self.root.join("expansion-settings.json"),
            65_536,
        ) {
            Err(crate::types::ServiceError::NotFound) => Ok(ProductSettings::default()),
            Ok(settings) if settings.validate() => Ok(settings),
            _ => Err(ProductError::Unavailable),
        }
    }
    pub fn save(&self, settings: &ProductSettings) -> Result<(), ProductError> {
        if !settings.validate() {
            return Err(ProductError::InvalidSettings);
        }
        let _guard = self.lock.lock().map_err(|_| ProductError::Unavailable)?;
        self.read()?;
        crate::utils::storage::write_json(
            &self.root.join("expansion-settings.json"),
            settings,
            65_536,
        )
        .map_err(|_| ProductError::Unavailable)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_settings_survive_independent_appearance_saves() {
        let root = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(root.path().into());
        let settings = ProductSettings {
            autostart: true,
            close_to_tray: false,
            toggle_shortcut: "Ctrl+Alt+T".into(),
            ..Default::default()
        };
        store.save(&settings).unwrap();
        crate::commands::preferences::PreferencesStore::new(root.path().into())
            .save_appearance(&crate::types::AppPreferences {
                theme: crate::types::Theme::Dark,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(store.load().unwrap(), settings);
    }
    #[test]
    fn mode_round_trips_and_corrupt_settings_are_preserved() {
        let directory = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(directory.path().into());
        let mut settings = store.load().unwrap();
        settings.insertion = super::super::settings::InsertionMode::Type;
        store.save(&settings).unwrap();
        assert_eq!(store.load().unwrap(), settings);
        let path = directory.path().join("expansion-settings.json");
        std::fs::write(&path, "{broken").unwrap();
        assert_eq!(store.load(), Err(ProductError::Unavailable));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "{broken");
    }
}
