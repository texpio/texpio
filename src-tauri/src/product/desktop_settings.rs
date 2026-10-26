//! Transactional desktop settings, tested without global registration or registry writes.
use super::{effects::ProductError, settings::ProductSettings};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DesktopAction {
    Toggle,
    Clipboard,
}
pub trait DesktopBackend {
    type Startup: Clone;
    fn canonicalize(&self, chord: &str) -> Result<String, ProductError>;
    fn register(&mut self, chord: &str) -> Result<(), ProductError>;
    fn unregister(&mut self, chord: &str) -> Result<(), ProductError>;
    fn startup_snapshot(&self) -> Result<Self::Startup, ProductError>;
    fn startup(&mut self, enabled: bool) -> Result<(), ProductError>;
    fn restore_startup(&mut self, value: Self::Startup) -> Result<(), ProductError>;
    fn save(&mut self, settings: &ProductSettings) -> Result<(), ProductError>;
    fn publish(&mut self, keys: &[(DesktopAction, String)]);
}
pub struct DesktopController<B: DesktopBackend> {
    backend: B,
    current: ProductSettings,
    keys: Vec<(DesktopAction, String)>,
    owned: HashSet<String>,
    repair_save: bool,
    repair_startup: Option<B::Startup>,
}
impl<B: DesktopBackend> DesktopController<B> {
    pub fn initialize_session(
        &mut self,
        settings: ProductSettings,
        start: impl FnOnce() -> Result<(), ProductError>,
    ) -> Result<(), ProductError> {
        if !settings.validate() {
            return Err(ProductError::InvalidSettings);
        }
        let desktop = self.initialize(settings);
        if desktop == Err(ProductError::InvalidSettings) {
            return desktop;
        }
        // Shortcut registration failure must not disable automatic expansion.
        let expansion = start();
        desktop.and(expansion)
    }
    pub fn shutdown(&mut self) {
        self.backend.publish(&[]);
        for key in self.owned.clone() {
            if self.backend.unregister(&key).is_ok() {
                self.owned.remove(&key);
            }
        }
    }
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            current: ProductSettings::default(),
            keys: vec![],
            owned: HashSet::new(),
            repair_save: false,
            repair_startup: None,
        }
    }
    pub fn initialize(&mut self, settings: ProductSettings) -> Result<(), ProductError> {
        self.current = settings.clone();
        let keys = self.plan(&settings)?;
        let mut added = Vec::new();
        for (_, key) in &keys {
            if let Err(error) = self.backend.register(key) {
                self.rollback(&added, &[], None, false);
                return Err(error);
            }
            self.owned.insert(key.clone());
            added.push(key.clone());
        }
        self.keys = keys;
        self.backend.publish(&self.keys);
        Ok(())
    }
    fn plan(
        &self,
        settings: &ProductSettings,
    ) -> Result<Vec<(DesktopAction, String)>, ProductError> {
        if !settings.validate() {
            return Err(ProductError::InvalidSettings);
        }
        let mut keys = vec![(
            DesktopAction::Toggle,
            self.backend.canonicalize(&settings.toggle_shortcut)?,
        )];
        if settings.clipboard_enabled {
            let key = self.backend.canonicalize(&settings.clipboard_shortcut)?;
            if keys[0].1 == key {
                return Err(ProductError::InvalidSettings);
            }
            keys.push((DesktopAction::Clipboard, key));
        }
        Ok(keys)
    }
    fn rollback(
        &mut self,
        added: &[String],
        removed: &[String],
        startup: Option<B::Startup>,
        saved: bool,
    ) {
        if let Some(snapshot) = startup
            && self.backend.restore_startup(snapshot.clone()).is_err()
        {
            self.repair_startup = Some(snapshot);
        }
        if saved && self.backend.save(&self.current).is_err() {
            self.repair_save = true;
        }
        for key in removed {
            if self.backend.register(key).is_ok() {
                self.owned.insert(key.clone());
            }
        }
        for key in added {
            if self.backend.unregister(key).is_ok() {
                self.owned.remove(key);
            }
        }
        let active: Vec<_> = self
            .keys
            .iter()
            .filter(|(_, key)| self.owned.contains(key))
            .cloned()
            .collect();
        self.backend.publish(&active);
    }
    pub fn apply(&mut self, settings: ProductSettings) -> Result<ProductSettings, ProductError> {
        let keys = self.plan(&settings)?;
        if let Some(snapshot) = self.repair_startup.clone() {
            self.backend.restore_startup(snapshot)?;
            self.repair_startup = None;
        }
        if self.repair_save {
            self.backend.save(&self.current)?;
            self.repair_save = false;
        }
        for key in self.owned.clone() {
            if !self.keys.iter().any(|(_, current)| current == &key) {
                self.backend.unregister(&key)?;
                self.owned.remove(&key);
            }
        }
        let mut added = Vec::new();
        let mut removed = Vec::new();
        for (_, key) in &keys {
            if !self.owned.contains(key) {
                if let Err(error) = self.backend.register(key) {
                    self.rollback(&added, &[], None, false);
                    return Err(error);
                }
                self.owned.insert(key.clone());
                added.push(key.clone());
            }
        }
        let startup = if settings.autostart != self.current.autostart {
            let snapshot = match self.backend.startup_snapshot() {
                Ok(value) => value,
                Err(error) => {
                    self.rollback(&added, &[], None, false);
                    return Err(error);
                }
            };
            if let Err(error) = self.backend.startup(settings.autostart) {
                self.rollback(&added, &[], Some(snapshot), false);
                return Err(error);
            }
            Some(snapshot)
        } else {
            None
        };
        if let Err(error) = self.backend.save(&settings) {
            self.rollback(&added, &[], startup, false);
            return Err(error);
        }
        for (_, key) in self.keys.clone() {
            if !keys.iter().any(|(_, desired)| desired == &key) {
                if let Err(error) = self.backend.unregister(&key) {
                    self.rollback(&added, &removed, startup, true);
                    return Err(error);
                }
                self.owned.remove(&key);
                removed.push(key);
            }
        }
        self.current = settings.clone();
        self.keys = keys;
        self.backend.publish(&self.keys);
        Ok(settings)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        collections::HashSet,
        sync::{Arc, Mutex},
    };
    #[derive(Default)]
    struct State {
        keys: HashSet<String>,
        published: Vec<(DesktopAction, String)>,
        startup: bool,
        saved: ProductSettings,
        conflict: Option<String>,
        invalid_chord: Option<String>,
        fail_save: bool,
        fail_unregister: Option<String>,
    }
    struct Fake(Arc<Mutex<State>>);
    impl DesktopBackend for Fake {
        type Startup = bool;
        fn canonicalize(&self, chord: &str) -> Result<String, ProductError> {
            if self.0.lock().unwrap().invalid_chord.as_deref() == Some(chord) {
                return Err(ProductError::InvalidSettings);
            }
            Ok(chord.to_ascii_lowercase())
        }
        fn register(&mut self, chord: &str) -> Result<(), ProductError> {
            let mut state = self.0.lock().unwrap();
            if state.conflict.as_deref() == Some(chord) {
                return Err(ProductError::Unavailable);
            }
            state.keys.insert(chord.into());
            Ok(())
        }
        fn unregister(&mut self, chord: &str) -> Result<(), ProductError> {
            let mut state = self.0.lock().unwrap();
            if state.fail_unregister.as_deref() == Some(chord) {
                return Err(ProductError::Unavailable);
            }
            state.keys.remove(chord);
            Ok(())
        }
        fn startup_snapshot(&self) -> Result<bool, ProductError> {
            Ok(self.0.lock().unwrap().startup)
        }
        fn startup(&mut self, value: bool) -> Result<(), ProductError> {
            self.0.lock().unwrap().startup = value;
            Ok(())
        }
        fn restore_startup(&mut self, value: bool) -> Result<(), ProductError> {
            self.startup(value)
        }
        fn save(&mut self, settings: &ProductSettings) -> Result<(), ProductError> {
            let mut state = self.0.lock().unwrap();
            if state.fail_save {
                state.fail_save = false;
                return Err(ProductError::Unavailable);
            }
            state.saved = settings.clone();
            Ok(())
        }
        fn publish(&mut self, keys: &[(DesktopAction, String)]) {
            self.0.lock().unwrap().published = keys.to_vec()
        }
    }
    fn fixture() -> (DesktopController<Fake>, Arc<Mutex<State>>) {
        let state = Arc::new(Mutex::new(State::default()));
        let mut controller = DesktopController::new(Fake(state.clone()));
        controller.initialize(ProductSettings::default()).unwrap();
        (controller, state)
    }
    #[test]
    fn every_session_starts_even_with_legacy_launch_disabled_without_rewriting_settings() {
        let state = Arc::new(Mutex::new(State::default()));
        let mut controller = DesktopController::new(Fake(state.clone()));
        let calls = std::cell::Cell::new(0);
        controller
            .initialize_session(
                ProductSettings {
                    enabled: false,
                    ..Default::default()
                },
                || {
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(state.lock().unwrap().saved, ProductSettings::default());
        assert!(!state.lock().unwrap().startup);
        controller.shutdown();
        let mut next = DesktopController::new(Fake(state));
        next.initialize_session(
            ProductSettings {
                enabled: false,
                ..Default::default()
            },
            || {
                calls.set(calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(calls.get(), 2);
    }
    #[test]
    fn native_shortcut_validation_refuses_startup_before_any_expansion() {
        let state = Arc::new(Mutex::new(State::default()));
        state.lock().unwrap().invalid_chord = Some("Space".into());
        let mut controller = DesktopController::new(Fake(state));
        let started = std::cell::Cell::new(false);
        let settings = ProductSettings {
            toggle_shortcut: "Space".into(),
            ..Default::default()
        };
        assert!(
            settings.validate(),
            "native parsing adds validation beyond the stored shape"
        );
        assert_eq!(
            controller.initialize_session(settings, || {
                started.set(true);
                Ok(())
            }),
            Err(ProductError::InvalidSettings)
        );
        assert!(
            !started.get(),
            "invalid native shortcut must not start expansion"
        );
    }
    #[test]
    fn invalid_settings_never_start_and_shortcut_conflicts_do_not_block_expansion() {
        let state = Arc::new(Mutex::new(State::default()));
        let mut controller = DesktopController::new(Fake(state.clone()));
        assert_eq!(
            controller.initialize_session(
                ProductSettings {
                    attachment_delay_ms: 0,
                    ..Default::default()
                },
                || panic!("invalid settings must not start")
            ),
            Err(ProductError::InvalidSettings)
        );
        state.lock().unwrap().conflict = Some(
            ProductSettings::default()
                .toggle_shortcut
                .to_ascii_lowercase(),
        );
        let started = std::cell::Cell::new(false);
        assert_eq!(
            controller.initialize_session(ProductSettings::default(), || {
                started.set(true);
                Ok(())
            }),
            Err(ProductError::Unavailable)
        );
        assert!(started.get());
    }
    #[test]
    fn conflict_preserves_old_hotkeys_and_disabling_clipboard_unregisters_it() {
        let (mut controller, state) = fixture();
        let original = state.lock().unwrap().keys.clone();
        assert_eq!(original.len(), 2);
        let mut settings = ProductSettings {
            toggle_shortcut: "Ctrl+Alt+T".into(),
            ..Default::default()
        };
        state.lock().unwrap().conflict = Some("ctrl+alt+t".into());
        assert!(controller.apply(settings.clone()).is_err());
        assert_eq!(state.lock().unwrap().keys, original);
        state.lock().unwrap().conflict = None;
        settings.clipboard_enabled = false;
        controller.apply(settings).unwrap();
        let state = state.lock().unwrap();
        assert_eq!(state.keys.len(), 1);
        assert_eq!(state.published.len(), 1);
        assert!(!state.saved.clipboard_enabled)
    }
    #[test]
    fn save_failure_rolls_back_hotkeys_and_exact_startup_state() {
        let (mut controller, state) = fixture();
        let original = state.lock().unwrap().keys.clone();
        assert_eq!(original.len(), 2);
        state.lock().unwrap().fail_save = true;
        let settings = ProductSettings {
            toggle_shortcut: "Ctrl+Alt+T".into(),
            autostart: true,
            ..Default::default()
        };
        assert!(controller.apply(settings).is_err());
        let state = state.lock().unwrap();
        assert!(!state.fail_save);
        assert_eq!(state.keys, original);
        assert!(!state.startup);
        assert!(!state.saved.autostart)
    }
    #[test]
    fn failed_old_key_removal_restores_saved_settings_before_retry() {
        let (mut controller, state) = fixture();
        let original = state.lock().unwrap().keys.clone();
        state.lock().unwrap().fail_unregister = Some(
            ProductSettings::default()
                .toggle_shortcut
                .to_ascii_lowercase(),
        );
        let settings = ProductSettings {
            toggle_shortcut: "Ctrl+Alt+T".into(),
            autostart: true,
            ..Default::default()
        };
        assert!(controller.apply(settings.clone()).is_err());
        {
            let state = state.lock().unwrap();
            assert_eq!(state.keys, original);
            assert_eq!(state.saved, ProductSettings::default());
            assert!(!state.startup);
        }
        state.lock().unwrap().fail_unregister = None;
        controller.apply(settings.clone()).unwrap();
        assert_eq!(state.lock().unwrap().saved, settings)
    }
}
