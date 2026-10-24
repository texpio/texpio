//! Content operations and native-session publication share one serialized boundary.
use crate::{
    engine::{
        rules::{Rule, RuleSet},
        runtime::ProbeHandle,
        types::{ProbePhase, ProbeStatus, RefusalReason},
    },
    library::{ImportPreview, Library, LibraryError, LibraryStore, SaveLibrary},
    probe_commands::ProbeError,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};

pub struct LibraryService {
    store: LibraryStore,
    control: Mutex<()>,
}

impl LibraryService {
    pub(crate) fn start_product(
        &self,
        engine: &crate::product::service::ProductEngine,
        settings: crate::product::settings::ProductSettings,
    ) -> Result<crate::product::service::ProductStatus, crate::product::effects::ProductError> {
        self.start_product_admitted(engine, settings, engine.admission())
    }
    pub(crate) fn start_product_admitted(
        &self,
        engine: &crate::product::service::ProductEngine,
        settings: crate::product::settings::ProductSettings,
        ticket: u64,
    ) -> Result<crate::product::service::ProductStatus, crate::product::effects::ProductError> {
        let _control = self
            .control
            .lock()
            .map_err(|_| crate::product::effects::ProductError::Busy)?;
        let library = self
            .store
            .load()
            .map_err(|_| crate::product::effects::ProductError::Unavailable)?;
        let rules = library
            .snippets
            .into_iter()
            .filter(|snippet| snippet.enabled && !snippet.deleted)
            .map(|snippet| crate::product::matcher::ProductRule {
                id: snippet.id,
                trigger: snippet.trigger,
                body: snippet.body.into(),
            })
            .collect();
        let usage = Arc::new(LibraryStore::new(self.store.root().to_path_buf()));
        engine.record_with(move |id| {
            let usage = usage.clone();
            let id = id.to_owned();
            tauri::async_runtime::spawn_blocking(move || {
                let _ = usage.record_usage(&id);
            });
        });
        engine.start_admitted(rules, settings, ticket)
    }
    pub fn new(root: PathBuf) -> Self {
        Self {
            store: LibraryStore::new(root),
            control: Mutex::new(()),
        }
    }

    fn save(&self, handle: &ProbeHandle, input: SaveLibrary) -> Result<Library, LibraryError> {
        let _control = self.control.lock().map_err(|_| storage_error())?;
        stopped(handle)?;
        self.store.save(input)
    }

    fn import(
        &self,
        handle: &ProbeHandle,
        json: &str,
        revision: u32,
    ) -> Result<Library, LibraryError> {
        let _control = self.control.lock().map_err(|_| storage_error())?;
        stopped(handle)?;
        self.store.import_json(json, revision)
    }

    fn start(&self, handle: &ProbeHandle) -> Result<ProbeStatus, RefusalReason> {
        let _control = self
            .control
            .lock()
            .map_err(|_| RefusalReason::ContextUnavailable)?;
        let library = self
            .store
            .load()
            .map_err(|_| RefusalReason::ContextUnavailable)?;
        handle.start_library(Arc::new(rule_snapshot(&library)?))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn library_metadata(
    service: State<'_, Arc<LibraryService>>,
) -> Result<Vec<crate::library::SnippetInfo>, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.metadata()).await
}
#[tauri::command]
#[specta::specta]
pub async fn library_set_favorite(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    id: String,
    favorite: bool,
) -> Result<(), LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.set_favorite(&id, favorite)).await?;
    let _ = app.emit("library-metadata-changed", ());
    Ok(())
}

fn storage_error() -> LibraryError {
    LibraryError {
        code: "storage".into(),
    }
}

fn stopped(handle: &ProbeHandle) -> Result<(), LibraryError> {
    if handle.invalidate_library().phase == ProbePhase::Stopped {
        Ok(())
    } else {
        Err(storage_error())
    }
}

fn rule_snapshot(library: &Library) -> Result<RuleSet, RefusalReason> {
    RuleSet::new(
        library.revision,
        library
            .snippets
            .iter()
            .filter(|item| item.enabled && !item.deleted)
            .map(|item| Rule {
                id: item.id.clone(),
                trigger: item.trigger.clone(),
                body: item.body.clone(),
            })
            .collect(),
    )
}

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, LibraryError> + Send + 'static,
) -> Result<T, LibraryError> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| storage_error())?
}

#[tauri::command]
#[specta::specta]
pub async fn library_load(
    service: State<'_, Arc<LibraryService>>,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.load()).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_save(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
    settings: State<'_, Arc<crate::product::store::SettingsStore>>,
    input: SaveLibrary,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    let handle = handle.inner().clone();
    let product = product.inner().clone();
    let settings = settings.inner().clone();
    let result = blocking(move || {
        let running = product.status().phase == crate::product::service::ProductPhase::Running;
        if product.stop().phase != crate::product::service::ProductPhase::Stopped {
            return Err(storage_error());
        }
        let result = service.save(&handle, input);
        if running {
            let _ = settings
                .load()
                .and_then(|settings| service.start_product(&product, settings));
        }
        result
    })
    .await;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    if result.is_ok() {
        let _ = app.emit("library-changed", ());
    }
    result
}

#[tauri::command]
#[specta::specta]
pub async fn library_export(
    service: State<'_, Arc<LibraryService>>,
) -> Result<String, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.export_json()).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_import_preview(
    service: State<'_, Arc<LibraryService>>,
    json: String,
) -> Result<ImportPreview, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.preview_import(&json)).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_import(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
    settings: State<'_, Arc<crate::product::store::SettingsStore>>,
    json: String,
    expected_revision: u32,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    let handle = handle.inner().clone();
    let product = product.inner().clone();
    let settings = settings.inner().clone();
    let result = blocking(move || {
        let running = product.status().phase == crate::product::service::ProductPhase::Running;
        if product.stop().phase != crate::product::service::ProductPhase::Stopped {
            return Err(storage_error());
        }
        let result = service.import(&handle, &json, expected_revision);
        if running {
            let _ = settings
                .load()
                .and_then(|settings| service.start_product(&product, settings));
        }
        result
    })
    .await;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    if result.is_ok() {
        let _ = app.emit("library-changed", ());
    }
    result
}

#[tauri::command]
#[specta::specta]
pub async fn library_start(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
) -> Result<ProbeStatus, ProbeError> {
    let service = service.inner().clone();
    let handle = handle.inner().clone();
    let product = product.inner().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        if product.stop().phase != crate::product::service::ProductPhase::Stopped {
            return Err(RefusalReason::Busy);
        }
        service.start(&handle)
    })
    .await
    .map_err(|_| ProbeError::from(RefusalReason::OutcomeUncertain))?;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    result.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_excludes_disabled_and_trashed_records() {
        let directory = tempfile::tempdir().unwrap();
        let service = LibraryService::new(directory.path().into());
        let mut library = service.store.load().unwrap();
        assert!(library.snippets.len() >= 2);
        let kept = library.snippets[0].id.clone();
        library.snippets[1].enabled = false;
        let mut trash = library.snippets[0].clone();
        trash.id = "00000000-0000-4000-8000-000000000099".into();
        trash.deleted = true;
        library.snippets.push(trash);
        let rules = rule_snapshot(&library).unwrap();
        assert_eq!(rules.entries().len(), 1);
        assert_eq!(rules.entries()[0].id, kept);
    }

    #[test]
    fn save_and_import_leave_observation_stopped_and_reject_stale_saves() {
        let directory = tempfile::tempdir().unwrap();
        let service = LibraryService::new(directory.path().into());
        let handle = ProbeHandle::new();
        let library = service.store.load().unwrap();
        let input = SaveLibrary {
            expected_revision: library.revision,
            folders: library.folders,
            snippets: library.snippets,
        };
        let mut changed = input.clone();
        changed.snippets[0].body = "Changed local body".into();
        let saved = service.save(&handle, changed).unwrap();
        assert_eq!(handle.status().phase, ProbePhase::Stopped);
        assert_eq!(service.save(&handle, input).unwrap_err().code, "conflict");
        let json = service.store.export_json().unwrap();
        assert_eq!(
            service.import(&handle, &json, saved.revision).unwrap(),
            saved
        );
        assert_eq!(handle.status().phase, ProbePhase::Stopped);
    }
}
