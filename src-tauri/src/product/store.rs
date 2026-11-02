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
        let settings = match crate::utils::storage::read_json::<ProductSettings>(
            &self.root.join("expansion-settings.json"),
            65_536,
        ) {
            Err(crate::types::ServiceError::NotFound) => ProductSettings::default(),
            Ok(settings) if settings.validate() => settings,
            _ => return Err(ProductError::Unavailable),
        };
        Ok(sanitize(settings))
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
/// macOS paste cannot restore the previous clipboard, so a stored paste+restore
/// pair would make every expansion start fail. Repair it when loading.
fn sanitize(mut settings: ProductSettings) -> ProductSettings {
    #[cfg(target_os = "macos")]
    if settings.insertion == super::settings::InsertionMode::Paste {
        settings.restore_clipboard = false;
    }
    settings
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paste_mode_never_keeps_clipboard_restoration_where_the_pasteboard_cannot_be_restored() {
        use crate::product::settings::InsertionMode;
        let settings = sanitize(ProductSettings {
            insertion: InsertionMode::Paste,
            restore_clipboard: true,
            ..Default::default()
        });
        #[cfg(target_os = "macos")]
        assert!(!settings.restore_clipboard);
        #[cfg(not(target_os = "macos"))]
        assert!(settings.restore_clipboard);
    }
    #[test]
    fn desktop_settings_survive_independent_appearance_saves() {
        let root = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(root.path().into());
        let settings = ProductSettings {
            autostart: true,
            close_to_tray: false,
            toggle_shortcut: "Ctrl+Alt+T".into(),
            insertion: crate::product::settings::InsertionMode::Type,
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
