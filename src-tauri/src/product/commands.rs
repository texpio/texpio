use super::{
    effects::ProductError,
    service::{ProductEngine, ProductStatus},
    settings::ProductSettings,
    store::SettingsStore,
};
use crate::{engine::runtime::ProbeHandle, library_commands::LibraryService};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
#[specta::specta]
pub fn product_status(engine: State<'_, Arc<ProductEngine>>) -> ProductStatus {
    engine.status()
}

#[tauri::command]
#[specta::specta]
pub async fn product_start(
    service: State<'_, Arc<LibraryService>>,
    engine: State<'_, Arc<ProductEngine>>,
    probe: State<'_, Arc<ProbeHandle>>,
    settings: State<'_, Arc<SettingsStore>>,
) -> Result<ProductStatus, ProductError> {
    let service = service.inner().clone();
    let engine = engine.inner().clone();
    let probe = probe.inner().clone();
    let settings = settings.inner().clone();
    let admission = engine.admission();
    tauri::async_runtime::spawn_blocking(move || {
        service.start_product_admitted(&engine, &probe, &settings, admission)
    })
    .await
    .map_err(|_| ProductError::Unavailable)?
}
#[tauri::command]
#[specta::specta]
pub async fn product_stop(
    engine: State<'_, Arc<ProductEngine>>,
    probe: State<'_, Arc<ProbeHandle>>,
) -> Result<ProductStatus, ProductError> {
    let engine = engine.inner().clone();
    let probe = probe.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = engine.stop();
        probe.stop();
        result
    })
    .await
    .map_err(|_| ProductError::Unavailable)
}
#[tauri::command]
#[specta::specta]
pub async fn product_pause(
    engine: State<'_, Arc<ProductEngine>>,
) -> Result<ProductStatus, ProductError> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.pause())
        .await
        .map_err(|_| ProductError::Unavailable)
}
#[tauri::command]
#[specta::specta]
pub async fn product_settings(
    settings: State<'_, Arc<SettingsStore>>,
) -> Result<ProductSettings, ProductError> {
    let settings = settings.inner().clone();
    tauri::async_runtime::spawn_blocking(move || settings.load())
        .await
        .map_err(|_| ProductError::Unavailable)?
}
#[tauri::command]
#[specta::specta]
pub async fn product_save_settings(
    app: AppHandle,
    settings: State<'_, Arc<SettingsStore>>,
    engine: State<'_, Arc<ProductEngine>>,
    service: State<'_, Arc<LibraryService>>,
    input: ProductSettings,
) -> Result<ProductSettings, ProductError> {
    let settings = settings.inner().clone();
    let engine = engine.inner().clone();
    let service = service.inner().clone();
    #[cfg(target_os = "windows")]
    let desktop = app
        .state::<super::desktop_native::DesktopSettingsActor>()
        .inner()
        .clone();
    let saved = tauri::async_runtime::spawn_blocking(move || {
        if !input.validate() {
            return Err(ProductError::InvalidSettings);
        }
        service.update_product(
            || engine.suspend_for_update(),
            || {
                #[cfg(target_os = "windows")]
                desktop.apply(input.clone())?;
                #[cfg(not(target_os = "windows"))]
                settings.save(&input)?;
                Ok(input)
            },
            |ticket| {
                service.resume_product_after_update(&engine, &settings, ticket);
            },
        )?
    })
    .await
    .map_err(|_| ProductError::Unavailable)??;
    let _ = app.emit("product-settings-changed", ());
    Ok(saved)
}
