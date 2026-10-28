use super::{LibraryService, blocking, stopped, storage_error};
use crate::{
    engine::runtime::ProbeHandle,
    library::{LibraryError, location::LibraryLocation},
    product::{service::ProductEngine, store::SettingsStore},
};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_fs::FsExt;

#[tauri::command]
#[specta::specta]
pub async fn library_location(
    service: State<'_, Arc<LibraryService>>,
) -> Result<LibraryLocation, LibraryError> {
    service.store.location()
}
#[tauri::command]
#[specta::specta]
pub async fn library_switch(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    probe: State<'_, Arc<ProbeHandle>>,
    settings: State<'_, Arc<SettingsStore>>,
    directory: Option<String>,
) -> Result<LibraryLocation, LibraryError> {
    let root = directory.map(PathBuf::from);
    if root
        .as_ref()
        .is_some_and(|path| !path.is_absolute() || !app.fs_scope().is_allowed(path))
    {
        return Err(LibraryError {
            code: "invalidData".into(),
        });
    }
    let service = service.inner().clone();
    let engine = engine.inner().clone();
    let probe = probe.inner().clone();
    let settings = settings.inner().clone();
    let result = blocking(move || {
        service
            .update_product(
                || engine.suspend_for_update(),
                || {
                    stopped(&probe)?;
                    service.store.switch(root)
                },
                |ticket| {
                    service.resume_product_after_update(&engine, &settings, ticket);
                },
            )
            .map_err(|_| storage_error())?
    })
    .await;
    if result.is_ok() {
        changed(&app);
    }
    result
}
fn changed(app: &AppHandle) {
    for event in [
        "library-changed",
        "library-metadata-changed",
        "library-files-changed",
        "library-location-changed",
    ] {
        let _ = app.emit(event, ());
    }
}
pub fn start_watch(app: &AppHandle, service: &Arc<LibraryService>) {
    let weak = Arc::downgrade(service);
    let app = app.clone();
    let _ = std::thread::Builder::new()
        .name("library-refresh".into())
        .spawn(move || {
            let mut current: Option<(PathBuf, crate::library::watch::LibraryWatch, u32)> = None;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let Some(service) = weak.upgrade() else {
                    return;
                };
                if !service.watching.load(Ordering::Acquire) {
                    return;
                }
                let Ok(store) = service.store.snapshot() else {
                    continue;
                };
                let root = store.root().to_path_buf();
                if current.as_ref().is_none_or(|(path, _, _)| path != &root) {
                    current = crate::library::watch::LibraryWatch::open(&root).and_then(|watch| {
                        store
                            .load()
                            .ok()
                            .map(|library| (root, watch, library.revision))
                    });
                    continue;
                }
                let Some((_, watch, revision)) = &mut current else {
                    continue;
                };
                if !watch.changed() {
                    continue;
                }
                if let Ok(library) = store.load()
                    && library.revision != *revision
                {
                    *revision = library.revision;
                    if let (Some(engine), Some(settings)) = (
                        app.try_state::<Arc<ProductEngine>>(),
                        app.try_state::<Arc<SettingsStore>>(),
                    ) {
                        let _ = service.coordinated(|| {
                            if service.published_revision.load(Ordering::Acquire)
                                == library.revision
                            {
                                return;
                            }
                            if let Ok(Some(ticket)) = engine.suspend_for_update() {
                                service.resume_product_after_update(&engine, &settings, ticket);
                            }
                        });
                    }
                }
                changed(&app);
            }
        });
}
pub fn stop_watch(service: &LibraryService) {
    service.watching.store(false, Ordering::Release);
}
