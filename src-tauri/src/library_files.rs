//! Attachment IPC: intake paths must have been granted by a native selection.
use super::{LibraryService, storage_error};
use crate::{
    library::{Attachment, FileSettings, LibraryError, LibraryStore},
    product::{service::ProductEngine, store::SettingsStore},
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_fs::FsExt;
#[derive(serde::Deserialize, specta::Type)]
#[serde(deny_unknown_fields)]
pub struct AttachmentInput {
    name: String,
    data: String,
}

fn selected_paths(
    paths: Vec<String>,
    allowed: impl Fn(&Path) -> bool,
) -> Result<Vec<PathBuf>, LibraryError> {
    if paths.is_empty() || paths.len() > 100 {
        return Err(LibraryError {
            code: "invalidData".into(),
        });
    }
    paths
        .into_iter()
        .map(|path| {
            if path.len() > 32768 || path.contains('\0') {
                return Err(LibraryError {
                    code: "invalidData".into(),
                });
            }
            let path = PathBuf::from(path);
            if !path.is_absolute() || !allowed(&path) {
                return Err(LibraryError {
                    code: "invalidData".into(),
                });
            }
            Ok(path)
        })
        .collect()
}
async fn mutate<T: Send + 'static>(
    app: AppHandle,
    service: Arc<LibraryService>,
    engine: Arc<ProductEngine>,
    settings: Arc<SettingsStore>,
    revision: u32,
    operation: impl FnOnce(&LibraryStore) -> Result<T, LibraryError> + Send + 'static,
) -> Result<T, LibraryError> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        let (result, resume) = {
            let _control = service.control.lock().map_err(|_| storage_error())?;
            let resume = engine.suspend_for_update().map_err(|_| storage_error())?;
            (service.store.with_revision(revision, operation), resume)
        };
        if let Some(ticket) = resume {
            let _ = settings
                .load()
                .and_then(|settings| service.start_product_admitted(&engine, settings, ticket));
        }
        result
    })
    .await
    .map_err(|_| storage_error())?;
    if result.is_ok() {
        let _ = app.emit("library-files-changed", ());
    }
    result
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_list(
    service: State<'_, Arc<LibraryService>>,
    id: String,
) -> Result<Vec<Attachment>, LibraryError> {
    let service = service.inner().clone();
    super::blocking(move || service.store.with(|store| store.list_attachments(&id))).await
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_settings(
    service: State<'_, Arc<LibraryService>>,
    id: String,
) -> Result<FileSettings, LibraryError> {
    let service = service.inner().clone();
    super::blocking(move || service.store.with(|store| store.file_settings(&id))).await
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_add(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    settings: State<'_, Arc<SettingsStore>>,
    id: String,
    revision: u32,
    paths: Vec<String>,
) -> Result<Vec<Attachment>, LibraryError> {
    let scope = app.fs_scope();
    let paths = selected_paths(paths, |path| scope.is_allowed(path))?;
    mutate(
        app,
        service.inner().clone(),
        engine.inner().clone(),
        settings.inner().clone(),
        revision,
        move |store| store.add_attachments(&id, &paths),
    )
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_add_data(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    settings: State<'_, Arc<SettingsStore>>,
    id: String,
    revision: u32,
    input: AttachmentInput,
) -> Result<Vec<Attachment>, LibraryError> {
    use base64::Engine;
    let AttachmentInput { name, data } = input;
    if data.len() > 8 * 1024 * 1024 {
        return Err(LibraryError {
            code: "tooLarge".into(),
        });
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&data)
        .map_err(|_| LibraryError {
            code: "invalidData".into(),
        })?;
    mutate(
        app,
        service.inner().clone(),
        engine.inner().clone(),
        settings.inner().clone(),
        revision,
        move |store| store.add_attachment_data(&id, &name, &bytes),
    )
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_remove(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    settings: State<'_, Arc<SettingsStore>>,
    id: String,
    revision: u32,
    attachment: String,
) -> Result<(), LibraryError> {
    mutate(
        app,
        service.inner().clone(),
        engine.inner().clone(),
        settings.inner().clone(),
        revision,
        move |store| store.remove_attachment(&id, &attachment),
    )
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_reorder(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    settings: State<'_, Arc<SettingsStore>>,
    id: String,
    revision: u32,
    order: Vec<String>,
) -> Result<(), LibraryError> {
    mutate(
        app,
        service.inner().clone(),
        engine.inner().clone(),
        settings.inner().clone(),
        revision,
        move |store| store.reorder_attachments(&id, &order),
    )
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn attachment_save_settings(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    settings: State<'_, Arc<SettingsStore>>,
    id: String,
    revision: u32,
    input: FileSettings,
) -> Result<(), LibraryError> {
    mutate(
        app,
        service.inner().clone(),
        engine.inner().clone(),
        settings.inner().clone(),
        revision,
        move |store| store.set_file_settings(&id, &input),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selected_file_scope_is_checked_for_every_path_before_intake() {
        let root = tempfile::tempdir().unwrap();
        let allowed = root.path().join("allowed.txt");
        let denied = root.path().join("denied.txt");
        assert_eq!(
            selected_paths(vec![allowed.to_string_lossy().into_owned()], |path| path
                == allowed)
            .unwrap(),
            vec![allowed.clone()]
        );
        assert!(
            selected_paths(
                vec![
                    allowed.to_string_lossy().into_owned(),
                    denied.to_string_lossy().into_owned()
                ],
                |path| path == allowed
            )
            .is_err()
        );
        assert!(selected_paths(vec!["relative.txt".into()], |_| true).is_err());
    }
}
