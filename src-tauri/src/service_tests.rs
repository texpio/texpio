use crate::{
    commands::{preferences::PreferencesStore, recovery::RecoveryStore},
    types::{AppPreferences, Language, MAX_RECOVERY_DATA_BYTES, ServiceError, Theme},
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

fn root() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.superpowers/sdd/2026-10-23-template-adoption/rust-fixtures")
        .join(format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn preferences_defaults_and_roundtrip() {
    let store = PreferencesStore::new(root());
    assert_eq!(store.load().unwrap(), AppPreferences::default());
    let preferences = AppPreferences {
        theme: Theme::Dark,
        language: Some(Language::Ar),
        quick_pane_shortcut: None,
    };
    store.save(&preferences).unwrap();
    assert_eq!(store.load().unwrap(), preferences);
}
#[test]
fn appearance_and_shortcut_writes_preserve_each_others_fields() {
    let store = PreferencesStore::new(root());
    store
        .save_shortcut(Some("CommandOrControl+Shift+J".into()))
        .unwrap();
    store
        .save_appearance(&AppPreferences {
            theme: Theme::Dark,
            language: Some(Language::Fr),
            quick_pane_shortcut: Some("stale chord".into()),
        })
        .unwrap();
    let saved = store.load().unwrap();
    assert_eq!(
        saved.quick_pane_shortcut.as_deref(),
        Some("CommandOrControl+Shift+J")
    );
    store
        .save_shortcut(Some("CommandOrControl+Shift+K".into()))
        .unwrap();
    let saved = store.load().unwrap();
    assert_eq!(saved.theme, Theme::Dark);
    assert_eq!(saved.language, Some(Language::Fr));
}

#[test]
fn legacy_theme_does_not_override_saved_system() {
    let store = PreferencesStore::new(root());
    store.save(&AppPreferences::default()).unwrap();
    store.migrate_legacy_theme(Theme::Dark).unwrap();
    assert_eq!(store.load().unwrap().theme, Theme::System);
}

#[test]
fn legacy_theme_migrates_only_a_missing_file() {
    let store = PreferencesStore::new(root());
    assert_eq!(
        store.migrate_legacy_theme(Theme::Light).unwrap().theme,
        Theme::Light
    );
    assert_eq!(
        store.migrate_legacy_theme(Theme::Dark).unwrap().theme,
        Theme::Light
    );
}

#[test]
fn corrupt_preferences_are_not_overwritten_by_migration() {
    let path = root();
    fs::write(path.join("preferences.json"), b"broken private path").unwrap();
    let store = PreferencesStore::new(path.clone());
    assert_eq!(store.load(), Err(ServiceError::CorruptData));
    assert_eq!(
        store.migrate_legacy_theme(Theme::Dark),
        Err(ServiceError::CorruptData)
    );
    assert_eq!(
        fs::read(path.join("preferences.json")).unwrap(),
        b"broken private path"
    );
}

#[test]
fn invalid_preferences_and_unknown_theme_refuse() {
    let store = PreferencesStore::new(root());
    let invalid = AppPreferences {
        quick_pane_shortcut: Some("\0".into()),
        ..AppPreferences::default()
    };
    assert_eq!(store.save(&invalid), Err(ServiceError::InvalidInput));
    assert!(serde_json::from_value::<AppPreferences>(json!({"language":"unsupported"})).is_err());
    assert!(
        serde_json::from_value::<AppPreferences>(
            json!({"theme":"future","language":null,"quick_pane_shortcut":null})
        )
        .is_err()
    );
}

#[test]
fn concurrent_preferences_saves_leave_complete_json() {
    let store = Arc::new(PreferencesStore::new(root()));
    let jobs: Vec<_> = (0..8)
        .map(|index| {
            let store = store.clone();
            std::thread::spawn(move || {
                store
                    .save(&AppPreferences {
                        theme: if index % 2 == 0 {
                            Theme::Light
                        } else {
                            Theme::Dark
                        },
                        ..AppPreferences::default()
                    })
                    .unwrap()
            })
        })
        .collect();
    for job in jobs {
        job.join().unwrap();
    }
    assert!(matches!(
        store.load().unwrap().theme,
        Theme::Light | Theme::Dark
    ));
}

#[cfg(windows)]
#[test]
fn failed_replace_keeps_previous_preferences_bytes() {
    let path = root();
    let store = PreferencesStore::new(path.clone());
    store.save(&AppPreferences::default()).unwrap();
    let file = path.join("preferences.json");
    let before = fs::read(&file).unwrap();
    let original_permissions = fs::metadata(&file).unwrap().permissions();
    let mut permissions = original_permissions.clone();
    permissions.set_readonly(true);
    fs::set_permissions(&file, permissions.clone()).unwrap();
    let result = store.save(&AppPreferences {
        theme: Theme::Dark,
        ..AppPreferences::default()
    });
    fs::set_permissions(&file, original_permissions).unwrap();
    assert_eq!(result, Err(ServiceError::Io));
    assert_eq!(fs::read(&file).unwrap(), before);
}

#[test]
fn recovery_rejects_traversal_reserved_names_and_oversized_data() {
    let store = RecoveryStore::new(root());
    for name in [
        "../escape",
        "a/b",
        "a\\b",
        "CON",
        "nul.txt",
        "COM1",
        "",
        ".",
    ] {
        assert_eq!(
            store.save(name, &json!({})),
            Err(ServiceError::InvalidInput)
        );
    }
    assert_eq!(
        store.save("large", &json!("x".repeat(MAX_RECOVERY_DATA_BYTES))),
        Err(ServiceError::DataTooLarge)
    );
}

#[test]
fn recovery_roundtrip_missing_and_oversized_load() {
    let path = root();
    let store = RecoveryStore::new(path.clone());
    assert_eq!(store.load("missing"), Err(ServiceError::NotFound));
    store.save("diagnostic", &json!({"event":"test"})).unwrap();
    assert_eq!(store.load("diagnostic").unwrap(), json!({"event":"test"}));
    fs::write(
        path.join("large.json"),
        vec![b' '; MAX_RECOVERY_DATA_BYTES + 1],
    )
    .unwrap();
    assert_eq!(store.load("large"), Err(ServiceError::DataTooLarge));
}

#[test]
fn recovery_cleanup_removes_only_old_json_files() {
    let path = root();
    let store = RecoveryStore::new(path.clone());
    store.save("old", &json!({})).unwrap();
    store.save("recent", &json!({})).unwrap();
    fs::write(path.join("keep.txt"), b"sentinel").unwrap();
    fs::OpenOptions::new()
        .write(true)
        .open(path.join("old.json"))
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(8 * 86400))
        .unwrap();
    assert_eq!(store.cleanup().unwrap(), 1);
    assert!(path.join("recent.json").exists());
    assert!(path.join("keep.txt").exists());
}

#[test]
fn service_errors_are_closed_codes_without_paths() {
    assert_eq!(serde_json::to_value(ServiceError::Io).unwrap(), json!("io"));
}

#[cfg(windows)]
#[test]
fn failed_recovery_finalization_keeps_the_previous_file() {
    let path = root();
    let store = RecoveryStore::new(path.clone());
    store
        .save("diagnostic", &json!({"event":"before"}))
        .unwrap();
    let file = path.join("diagnostic.json");
    let original = fs::metadata(&file).unwrap().permissions();
    let mut readonly = original.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&file, readonly).unwrap();
    let result = store.save("diagnostic", &json!({"event":"after"}));
    fs::set_permissions(&file, original).unwrap();
    assert_eq!(result, Err(ServiceError::Io));
    assert_eq!(store.load("diagnostic").unwrap(), json!({"event":"before"}));
}
