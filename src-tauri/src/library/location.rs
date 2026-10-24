use super::{ImportPreview, Library, LibraryError, LibraryStore, SaveLibrary, SnippetInfo};
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
        let (root, shared, mut problem) = match selection {
            Ok(Selection {
                directory: Some(root),
            }) => (root, true, false),
            Ok(_) | Err(crate::types::ServiceError::NotFound) => (personal.clone(), false, false),
            Err(_) => (personal.clone(), false, true),
        };
        if shared && !local_directory(&root) {
            problem = true;
        }
        let store = Arc::new(LibraryStore::new(root));
        if shared && !problem && store.load().is_err() {
            problem = true;
        }
        Self {
            personal,
            active: RwLock::new(Active {
                store,
                shared,
                problem,
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
    if !path.is_absolute() || !path.is_dir() {
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
    true
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
}
