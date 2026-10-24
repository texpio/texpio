use super::{
    effects::ProductError,
    service::{ProductEngine, ProductPhase, ProductStatus},
    settings::ProductSettings,
    store::SettingsStore,
};
use crate::{engine::runtime::ProbeHandle, library_commands::LibraryService};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

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
        if probe.stop().phase != crate::engine::types::ProbePhase::Stopped {
            return Err(ProductError::Busy);
        }
        service.start_product_admitted(&engine, settings.load()?, admission)
    })
    .await
    .map_err(|_| ProductError::Unavailable)?
}
#[tauri::command]
#[specta::specta]
pub async fn product_stop(
    engine: State<'_, Arc<ProductEngine>>,
) -> Result<ProductStatus, ProductError> {
    let engine = engine.inner().clone();
    tauri::async_runtime::spawn_blocking(move || engine.stop())
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
    let saved = tauri::async_runtime::spawn_blocking(move || {
        if !input.validate() {
            return Err(ProductError::InvalidSettings);
        }
        let running = engine.status().phase == ProductPhase::Running;
        if engine.stop().phase != ProductPhase::Stopped {
            return Err(ProductError::Busy);
        }
        settings.save(&input)?;
        if running {
            service.start_product(&engine, input.clone())?;
        }
        Ok(input)
    })
    .await
    .map_err(|_| ProductError::Unavailable)??;
    let _ = app.emit("product-settings-changed", ());
    Ok(saved)
}
