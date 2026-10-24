use crate::engine::{
    fixtures::{Fixture, fixtures},
    runtime::{ProbeHandle, StartOptions},
    types::{ProbeStatus, ReferenceProfile, RefusalReason},
};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Debug, Serialize, specta::Type)]
pub struct ProbeError {
    code: RefusalReason,
}
impl From<RefusalReason> for ProbeError {
    fn from(code: RefusalReason) -> Self {
        Self { code }
    }
}

fn supported() -> Result<(), ProbeError> {
    if cfg!(target_os = "windows") {
        Ok(())
    } else {
        Err(RefusalReason::UnsupportedPlatform.into())
    }
}

#[tauri::command]
#[specta::specta]
pub fn probe_status(
    app: AppHandle,
    handle: State<'_, Arc<ProbeHandle>>,
) -> Result<ProbeStatus, ProbeError> {
    supported()?;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    Ok(handle.status())
}

#[tauri::command]
#[specta::specta]
pub fn probe_fixtures() -> Result<Vec<Fixture>, ProbeError> {
    supported()?;
    Ok(fixtures())
}

async fn control(
    app: AppHandle,
    handle: Arc<ProbeHandle>,
    operation: impl FnOnce(&ProbeHandle) -> Result<ProbeStatus, RefusalReason> + Send + 'static,
) -> Result<ProbeStatus, ProbeError> {
    supported()?;
    let result = tauri::async_runtime::spawn_blocking(move || operation(&handle))
        .await
        .map_err(|_| ProbeError::from(RefusalReason::OutcomeUncertain))?;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    result.map_err(Into::into)
}

#[tauri::command]
#[specta::specta]
pub async fn probe_start(
    app: AppHandle,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
    options: StartOptions,
) -> Result<ProbeStatus, ProbeError> {
    let product = product.inner().clone();
    let stopped = tauri::async_runtime::spawn_blocking(move || product.stop())
        .await
        .map_err(|_| ProbeError::from(RefusalReason::Busy))?;
    if stopped.phase != crate::product::service::ProductPhase::Stopped {
        return Err(RefusalReason::Busy.into());
    }
    control(app, handle.inner().clone(), move |handle| {
        handle.start(options)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn probe_stop(
    app: AppHandle,
    handle: State<'_, Arc<ProbeHandle>>,
) -> Result<ProbeStatus, ProbeError> {
    control(app, handle.inner().clone(), |handle| Ok(handle.stop())).await
}
#[tauri::command]
#[specta::specta]
pub async fn probe_pause(
    app: AppHandle,
    handle: State<'_, Arc<ProbeHandle>>,
    paused: bool,
) -> Result<ProbeStatus, ProbeError> {
    control(app, handle.inner().clone(), move |handle| {
        handle.set_paused(paused)
    })
    .await
}
#[tauri::command]
#[specta::specta]
pub async fn probe_exclude(
    app: AppHandle,
    handle: State<'_, Arc<ProbeHandle>>,
    profile: ReferenceProfile,
) -> Result<ProbeStatus, ProbeError> {
    control(app, handle.inner().clone(), move |handle| {
        Ok(handle.exclude(profile))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{runtime::StartOptions, types::RefusalReason};

    #[test]
    fn errors_serialize_only_a_closed_code() {
        assert_eq!(
            serde_json::to_value(ProbeError::from(RefusalReason::PasswordField)).unwrap(),
            serde_json::json!({ "code": "passwordField" })
        );
    }
    #[test]
    fn start_options_reject_arbitrary_native_targets() {
        assert!(
            serde_json::from_value::<StartOptions>(serde_json::json!({
            "profile": "notepad", "strategy": "unicode", "windowHandle": 42 }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<StartOptions>(serde_json::json!({
            "profile": "other.exe", "strategy": "unicode" }))
            .is_err()
        );
    }
}
