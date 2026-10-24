use super::{
    ImportPreview, Library, LibraryError, LibraryStore, RecoveryStatus, SaveLibrary, SnippetInfo,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Selection {
    directory: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LibraryLocation {
    pub shared: bool,
    pub directory: String,
    pub problem: bool,
}
struct Active {
    store: Arc<LibraryStore>,
    shared: bool,
    problem: bool,
    selection_valid: bool,
}
pub struct LibraryRouter {
    personal: PathBuf,
    active: RwLock<Active>,
}
impl LibraryRouter {
    pub fn with_revision<T>(
        &self,
        revision: u32,
        operation: impl FnOnce(&LibraryStore) -> Result<T, LibraryError>,
    ) -> Result<T, LibraryError> {
        self.with(|store| {
            if store.load()?.revision != revision {
                return Err(LibraryError::conflict());
            }
            operation(store)
        })
    }
    pub fn new(personal: PathBuf) -> Self {
        let selection = crate::utils::storage::read_json::<Selection>(
            &personal.join("library-location.json"),
            65536,
        );
        let (root, shared, mut problem, selection_valid) = match selection {
            Ok(Selection {
                directory: Some(root),
            }) => (root, true, false, true),
            Ok(_) | Err(crate::types::ServiceError::NotFound) => {
                (personal.clone(), false, false, true)
            }
            Err(_) => (personal.clone(), false, true, false),
        };
        if shared && !local_directory(&root) {
            problem = true;
        }
        let store = Arc::new(LibraryStore::new(root));
        if shared && !problem {
            // Do not open a verified corrupt database for normal initialization:
            // SQLite may otherwise remove its WAL/SHM before explicit recovery.
            problem = match store.recovery_status() {
                Ok(status) if !status.can_recover => store.load().is_err(),
                _ => true,
            };
        }
        Self {
            personal,
            active: RwLock::new(Active {
                store,
                shared,
                problem,
                selection_valid,
            }),
        }
    }
    pub fn with<T>(
        &self,
        operation: impl FnOnce(&LibraryStore) -> Result<T, LibraryError>,
    ) -> Result<T, LibraryError> {
        let active = self.active.read().map_err(|_| LibraryError::storage())?;
        if active.problem {
            return Err(LibraryError::storage());
        }
        operation(&active.store)
    }
    pub fn snapshot(&self) -> Result<Arc<LibraryStore>, LibraryError> {
        let active = self.active.read().map_err(|_| LibraryError::storage())?;
        if active.problem {
            return Err(LibraryError::storage());
        }
        Ok(active.store.clone())
    }
    fn recovery_store(active: &Active) -> Result<&LibraryStore, LibraryError> {
        let root = active.store.root();
        if !active.selection_valid || !local_directory(root) {
            return Err(LibraryError::storage());
        }
        let resolved = root.canonicalize().map_err(|_| LibraryError::storage())?;
        if !local_directory(&resolved) {
            return Err(LibraryError::storage());
        }
        Ok(&active.store)
    }
    pub fn recovery_status(&self) -> Result<RecoveryStatus, LibraryError> {
        let active = self.active.read().map_err(|_| LibraryError::storage())?;
        Self::recovery_store(&active)?.recovery_status()
    }
    pub fn recover_corrupt(&self, confirmed: bool) -> Result<Library, LibraryError> {
        let mut active = self.active.write().map_err(|_| LibraryError::storage())?;
        let recovered = Self::recovery_store(&active)?.recover_corrupt(confirmed)?;
        active.problem = false;
        Ok(recovered)
    }
    pub fn location(&self) -> Result<LibraryLocation, LibraryError> {
        let active = self.active.read().map_err(|_| LibraryError::storage())?;
        Ok(LibraryLocation {
            shared: active.shared,
            directory: active.store.root().to_string_lossy().into_owned(),
            problem: active.problem,
        })
    }
    pub fn switch(&self, root: Option<PathBuf>) -> Result<LibraryLocation, LibraryError> {
        let shared = root.is_some();
        let root = root.unwrap_or_else(|| self.personal.clone());
        if !local_directory(&root) {
            return Err(LibraryError::invalid_data());
        }
        let root = root.canonicalize().map_err(|_| LibraryError::storage())?;
        let mut active = self.active.write().map_err(|_| LibraryError::storage())?;
        let minimum = if active.problem {
            0
        } else {
            active.store.load()?.revision
        };
        let store = Arc::new(LibraryStore::new(root.clone()));
        store.load()?;
        store.fence_revision(minimum)?;
        crate::utils::storage::write_json(
            &self.personal.join("library-location.json"),
            &Selection {
                directory: shared.then_some(root.clone()),
            },
            65536,
        )
        .map_err(|_| LibraryError::storage())?;
        *active = Active {
            store,
            shared,
            problem: false,
            selection_valid: true,
        };
        Ok(LibraryLocation {
            shared,
            directory: root.to_string_lossy().into_owned(),
            problem: false,
        })
    }
    pub fn load(&self) -> Result<Library, LibraryError> {
        self.with(LibraryStore::load)
    }
    pub fn save(&self, input: SaveLibrary) -> Result<Library, LibraryError> {
        self.with(|store| store.save(input))
    }
    pub fn import_json(&self, json: &str, revision: u32) -> Result<Library, LibraryError> {
        self.with(|store| store.import_json(json, revision))
    }
    pub fn export_json(&self) -> Result<String, LibraryError> {
        self.with(LibraryStore::export_json)
    }
    pub fn preview_import(&self, json: &str) -> Result<ImportPreview, LibraryError> {
        self.with(|store| store.preview_import(json))
    }
    pub fn metadata(&self) -> Result<Vec<SnippetInfo>, LibraryError> {
        self.with(LibraryStore::metadata)
    }
    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<(), LibraryError> {
        self.with(|store| store.set_favorite(id, favorite))
    }
}

fn local_directory(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    #[cfg(target_os = "windows")]
    {
        use std::{os::windows::ffi::OsStrExt, path::Component};
        let mut components = path.components();
        let Some(Component::Prefix(prefix)) = components.next() else {
            return false;
        };
        if !matches!(
            prefix.kind(),
            std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)
        ) {
            return false;
        }
        let mut drive = prefix.as_os_str().to_os_string();
        drive.push("\\");
        let wide: Vec<u16> = drive.encode_wide().chain(Some(0)).collect();
        let kind = unsafe {
            windows::Win32::Storage::FileSystem::GetDriveTypeW(windows::core::PCWSTR(wide.as_ptr()))
        };
        if !matches!(kind, 2 | 3 | 6) {
            return false;
        }
    }
    path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::SaveLibrary;
    #[test]
    fn switching_preserves_personal_content_and_fences_old_editor_revisions() {
        let personal = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let router = LibraryRouter::new(personal.path().into());
        let old = router.with(LibraryStore::load).unwrap();
        router.switch(Some(shared.path().into())).unwrap();
        let current = router.with(LibraryStore::load).unwrap();
        assert!(current.revision > old.revision);
        let stale = SaveLibrary {
            expected_revision: old.revision,
            folders: old.folders.clone(),
            snippets: old.snippets.clone(),
        };
        assert_eq!(
            router.with(|store| store.save(stale)).unwrap_err().code,
            "conflict"
        );
        router.switch(None).unwrap();
        let restored = router.with(LibraryStore::load).unwrap();
        assert_eq!(restored.snippets, old.snippets);
        assert_eq!(restored.folders, old.folders);
        assert!(restored.revision > current.revision);
    }
    #[test]
    fn failed_selection_preserves_active_library_and_persisted_selection() {
        let personal = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let bad = tempfile::NamedTempFile::new().unwrap();
        let router = LibraryRouter::new(personal.path().into());
        router.switch(Some(shared.path().into())).unwrap();
        let before = router.with(LibraryStore::load).unwrap();
        assert!(router.switch(Some(bad.path().into())).is_err());
        assert_eq!(router.with(LibraryStore::load).unwrap(), before);
        let reopened = LibraryRouter::new(personal.path().into());
        assert_eq!(reopened.with(LibraryStore::load).unwrap(), before);
    }
    #[test]
    fn pending_attachment_mutation_cannot_cross_a_library_switch() {
        let personal = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let router = LibraryRouter::new(personal.path().into());
        let old = router.load().unwrap();
        router.switch(Some(shared.path().into())).unwrap();
        let mut ran = false;
        let result = router.with_revision(old.revision, |_| {
            ran = true;
            Ok(())
        });
        assert_eq!(result.unwrap_err().code, "conflict");
        assert!(!ran);
    }
    #[test]
    fn selected_corrupt_library_can_be_recovered_after_restart_without_bypassing_confirmation() {
        let personal = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let router = LibraryRouter::new(personal.path().into());
        let personal_before = router.load().unwrap();
        router.switch(Some(shared.path().into())).unwrap();
        drop(router);
        let selected_before = std::fs::read(personal.path().join("library-location.json")).unwrap();
        let originals = [
            (
                "library.sqlite3",
                b"owned corrupt selected database".as_slice(),
            ),
            ("library.sqlite3-wal", b"owned selected wal"),
            ("library.sqlite3-shm", b"owned selected shm"),
        ];
        for (name, bytes) in originals {
            std::fs::write(shared.path().join(name), bytes).unwrap();
        }
        let reopened = LibraryRouter::new(personal.path().into());
        assert!(reopened.location().unwrap().problem);
        assert!(reopened.load().is_err());
        assert!(reopened.snapshot().is_err());
        assert!(
            reopened
                .recovery_status()
                .expect("selected corruption must remain visible to recovery")
                .can_recover
        );
        assert!(reopened.recover_corrupt(false).is_err());
        assert!(reopened.location().unwrap().problem);
        assert!(!shared.path().join("recovery").exists());
        for (name, bytes) in originals {
            assert_eq!(std::fs::read(shared.path().join(name)).unwrap(), bytes);
        }
        let recovered = reopened
            .recover_corrupt(true)
            .expect("confirmed corruption should recover the selected library");
        assert!(!reopened.location().unwrap().problem);
        assert_eq!(reopened.load().unwrap(), recovered);
        assert!(reopened.snapshot().is_ok());
        assert!(reopened.location().unwrap().shared);
        assert_eq!(
            std::fs::read(personal.path().join("library-location.json")).unwrap(),
            selected_before
        );
        let preserved = std::fs::read_dir(shared.path().join("recovery"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(preserved.len(), 1);
        for (name, bytes) in originals {
            assert_eq!(std::fs::read(preserved[0].join(name)).unwrap(), bytes);
        }
        assert_eq!(
            LibraryStore::new(personal.path().into()).load().unwrap(),
            personal_before
        );
    }
    #[test]
    fn recovery_refuses_invalid_or_missing_selected_roots_without_creating_them() {
        let personal = tempfile::tempdir().unwrap();
        let missing = personal.path().join("missing-selected-library");
        let not_directory = tempfile::NamedTempFile::new().unwrap();
        let mut roots = vec![
            missing.clone(),
            not_directory.path().into(),
            PathBuf::from("relative-library"),
        ];
        #[cfg(target_os = "windows")]
        roots.push(PathBuf::from(r"\\server\share\library"));
        for root in roots {
            std::fs::write(
                personal.path().join("library-location.json"),
                serde_json::to_vec(&Selection {
                    directory: Some(root),
                })
                .unwrap(),
            )
            .unwrap();
            let router = LibraryRouter::new(personal.path().into());
            assert!(router.location().unwrap().problem);
            assert!(router.recovery_status().is_err());
            assert!(router.recover_corrupt(true).is_err());
            assert!(router.location().unwrap().problem);
            assert!(router.snapshot().is_err());
        }
        assert!(!missing.exists());
        assert!(!personal.path().join("recovery").exists());
    }
    #[test]
    fn invalid_selection_cannot_recover_an_unselected_corrupt_personal_database() {
        let personal = tempfile::tempdir().unwrap();
        std::fs::write(personal.path().join("library-location.json"), b"{").unwrap();
        let database = personal.path().join("library.sqlite3");
        std::fs::write(&database, b"owned corrupt personal database").unwrap();
        let router = LibraryRouter::new(personal.path().into());
        assert!(router.location().unwrap().problem);
        assert!(!router.location().unwrap().shared);
        assert!(router.recovery_status().is_err());
        assert!(router.recover_corrupt(true).is_err());
        assert!(router.location().unwrap().problem);
        assert_eq!(
            std::fs::read(database).unwrap(),
            b"owned corrupt personal database"
        );
        assert!(!personal.path().join("recovery").exists());
    }
    #[test]
    fn selected_schema_and_io_failures_do_not_become_recovery_candidates() {
        let personal = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let router = LibraryRouter::new(personal.path().into());
        router.switch(Some(shared.path().into())).unwrap();
        drop(router);
        let database = shared.path().join("library.sqlite3");
        let connection = rusqlite::Connection::open(&database).unwrap();
        connection.execute_batch("PRAGMA user_version=999").unwrap();
        drop(connection);
        let reopened = LibraryRouter::new(personal.path().into());
        assert!(reopened.location().unwrap().problem);
        assert!(!reopened.recovery_status().unwrap().can_recover);
        assert!(reopened.recover_corrupt(true).is_err());
        assert!(reopened.location().unwrap().problem);
        assert!(!shared.path().join("recovery").exists());
        drop(reopened);
        std::fs::remove_file(database).unwrap();
        std::fs::create_dir(shared.path().join("library.sqlite3")).unwrap();
        let reopened = LibraryRouter::new(personal.path().into());
        assert!(reopened.location().unwrap().problem);
        assert!(reopened.recovery_status().is_err());
        assert!(reopened.recover_corrupt(true).is_err());
        assert!(reopened.location().unwrap().problem);
        assert!(!shared.path().join("recovery").exists());
    }
}
