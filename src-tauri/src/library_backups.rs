//! Selected files stay in Rust; previews return counts and a one-shot token.
use super::{LibraryService, blocking, stopped, storage_error};
use crate::{
    engine::runtime::ProbeHandle,
    library::{BackupFormat, ImportMode, Library, LibraryError, PreparedImport, RecoveryStatus},
    product::{service::ProductEngine, store::SettingsStore},
};
use std::{io::Read, path::PathBuf, sync::Arc, time::Instant};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_fs::FsExt;
const MAX_FILE: usize = 536 * 1024 * 1024;
pub(crate) struct StagedBackup {
    token: String,
    revision: u32,
    root: PathBuf,
    mode: ImportMode,
    prepared: PreparedImport,
    created: Instant,
}
#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BackupPreview {
    token: String,
    revision: u32,
    folders: u32,
    snippets: u32,
    skipped: u32,
    files: u32,
    replace: bool,
}
fn selected(app: &AppHandle, path: String) -> Result<PathBuf, LibraryError> {
    let path = PathBuf::from(path);
    if !path.is_absolute() || !app.fs_scope().is_allowed(&path) {
        return Err(LibraryError {
            code: "invalidData".into(),
        });
    }
    Ok(path)
}
fn notify(app: &AppHandle) {
    for event in [
        "library-changed",
        "library-files-changed",
        "library-metadata-changed",
    ] {
        let _ = app.emit(event, ());
    }
}
#[tauri::command]
#[specta::specta]
pub async fn backup_export(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    path: String,
    format: BackupFormat,
) -> Result<(), LibraryError> {
    let path = selected(&app, path)?;
    let service = service.inner().clone();
    blocking(move || {
        let bytes = service.store.with(|store| store.export_backup(format))?;
        crate::utils::storage::write_bytes(&path, &bytes, MAX_FILE).map_err(|_| storage_error())
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn backup_preview(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    path: String,
    format: BackupFormat,
    mode: ImportMode,
) -> Result<BackupPreview, LibraryError> {
    let path = selected(&app, path)?;
    let service = service.inner().clone();
    blocking(move || {
        let metadata = std::fs::symlink_metadata(&path).map_err(|_| storage_error())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(storage_error());
        }
        if metadata.len() > MAX_FILE as u64 {
            return Err(LibraryError {
                code: "tooLarge".into(),
            });
        }
        let mut data = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| storage_error())?
            .take(MAX_FILE as u64 + 1)
            .read_to_end(&mut data)
            .map_err(|_| storage_error())?;
        let prepared = PreparedImport::decode(&data, format)?;
        let (preview, revision, root) = service.store.with(|store| {
            let (preview, revision) = prepared.preview(store, mode)?;
            Ok((preview, revision, store.root().to_path_buf()))
        })?;
        let token = uuid::Uuid::new_v4().to_string();
        let files = prepared.file_count();
        *service.staged_backup.lock().map_err(|_| storage_error())? = Some(StagedBackup {
            token: token.clone(),
            revision,
            root,
            mode,
            prepared,
            created: Instant::now(),
        });
        Ok(BackupPreview {
            token,
            revision,
            files,
            folders: preview.folders,
            snippets: preview.snippets,
            skipped: preview.skipped,
            replace: matches!(mode, ImportMode::Replace),
        })
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn backup_discard(
    service: State<'_, Arc<LibraryService>>,
    token: String,
) -> Result<(), LibraryError> {
    let mut staged = service.staged_backup.lock().map_err(|_| storage_error())?;
    if staged.as_ref().is_some_and(|stage| stage.token == token) {
        *staged = None;
    }
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub async fn backup_import(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    probe: State<'_, Arc<ProbeHandle>>,
    settings: State<'_, Arc<SettingsStore>>,
    token: String,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    let engine = engine.inner().clone();
    let probe = probe.inner().clone();
    let settings = settings.inner().clone();
    let result = blocking(move || {
        let stage = {
            let mut staged = service.staged_backup.lock().map_err(|_| storage_error())?;
            if !staged.as_ref().is_some_and(|stage| {
                stage.token == token && stage.created.elapsed().as_secs() < 900
            }) {
                return Err(LibraryError {
                    code: "invalidData".into(),
                });
            }
            staged.take().ok_or_else(storage_error)?
        };
        service
            .update_product(
                || engine.suspend_for_update(),
                || {
                    stopped(&probe)?;
                    service.store.with_revision(stage.revision, |store| {
                        if store.root() != stage.root {
                            return Err(LibraryError {
                                code: "conflict".into(),
                            });
                        }
                        stage.prepared.apply(store, stage.revision, stage.mode)
                    })
                },
                |ticket| {
                    let _ = service.resume_product_locked(&engine, &settings, ticket);
                },
            )
            .map_err(|_| storage_error())?
    })
    .await;
    if result.is_ok() {
        notify(&app);
    }
    result
}
#[tauri::command]
#[specta::specta]
pub async fn library_recovery_status(
    service: State<'_, Arc<LibraryService>>,
) -> Result<RecoveryStatus, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.recovery_status()).await
}
#[tauri::command]
#[specta::specta]
pub async fn library_recover(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    probe: State<'_, Arc<ProbeHandle>>,
    confirmed: bool,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    let engine = engine.inner().clone();
    let probe = probe.inner().clone();
    let result = blocking(move || {
        let _guard = service.control.lock().map_err(|_| storage_error())?;
        if engine.stop().phase != crate::product::service::ProductPhase::Stopped {
            return Err(storage_error());
        }
        stopped(&probe)?;
        service.store.recover_corrupt(confirmed)
    })
    .await;
    if result.is_ok() {
        notify(&app);
    }
    result
}
