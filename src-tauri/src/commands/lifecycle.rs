use crate::engine::{
    runtime::ProbeHandle,
    types::{ProbePhase, RefusalReason},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Clone, Copy, Serialize, Type)]
pub struct UpdateConfiguration {
    pub configured: bool,
}
pub fn valid_update_configuration(value: &Value) -> bool {
    let Some(key) = value.get("pubkey").and_then(Value::as_str) else {
        return false;
    };
    if key.len() > 4096 {
        return false;
    }
    let Ok(bytes) = STANDARD.decode(key) else {
        return false;
    };
    let Ok(key) = std::str::from_utf8(&bytes) else {
        return false;
    };
    if minisign_verify::PublicKey::decode(key).is_err() {
        return false;
    }
    let Some(endpoints) = value.get("endpoints").and_then(Value::as_array) else {
        return false;
    };
    !endpoints.is_empty()
        && endpoints.len() <= 8
        && endpoints.iter().all(|endpoint| {
            let Some(endpoint) = endpoint.as_str() else {
                return false;
            };
            let lower = endpoint.to_ascii_lowercase();
            if endpoint.len() > 4096 || lower.contains("your_") || lower.contains("your-") {
                return false;
            }
            let Ok(url) = tauri::Url::parse(endpoint) else {
                return false;
            };
            url.scheme() == "https"
                && url.username().is_empty()
                && url.password().is_none()
                && url.fragment().is_none()
                && url.host_str().is_some_and(|host| {
                    !matches!(
                        host,
                        "localhost" | "example.com" | "example.org" | "example.net"
                    ) && !host.ends_with(".invalid")
                        && !host.ends_with(".example")
                })
        })
}
pub fn confirm_stopped(phase: ProbePhase) -> Result<(), RefusalReason> {
    if phase == ProbePhase::Stopped {
        Ok(())
    } else {
        Err(RefusalReason::OutcomeUncertain)
    }
}
#[tauri::command]
#[specta::specta]
pub fn get_update_configuration(
    configuration: State<'_, UpdateConfiguration>,
) -> UpdateConfiguration {
    *configuration
}
#[tauri::command]
#[specta::specta]
pub async fn prepare_for_relaunch(
    app: AppHandle,
    handle: State<'_, Arc<ProbeHandle>>,
) -> Result<(), RefusalReason> {
    let handle = handle.inner().clone();
    let phase = tauri::async_runtime::spawn_blocking(move || {
        handle.stop();
        handle.status().phase
    })
    .await
    .map_err(|_| RefusalReason::OutcomeUncertain)?;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    confirm_stopped(phase)
}

#[derive(Clone, Copy, Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticEvent {
    AppStarted,
    RenderFailed,
    CommandFailed,
    NotificationFailed,
    RecoverySaved,
    RecoveryFailed,
    UpdateFailed,
    Unknown,
}
#[derive(Clone, Copy, Deserialize, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
#[derive(Deserialize, Serialize, Type)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticRecord {
    pub level: DiagnosticLevel,
    pub event: DiagnosticEvent,
    pub count: Option<u32>,
}
#[tauri::command]
#[specta::specta]
pub fn record_diagnostic(record: DiagnosticRecord) {
    let text = serde_json::to_string(&record).unwrap_or_else(|_| "diagnosticUnavailable".into());
    match record.level {
        DiagnosticLevel::Trace => log::trace!(target: "texpio::diagnostic", "{text}"),
        DiagnosticLevel::Debug => log::debug!(target: "texpio::diagnostic", "{text}"),
        DiagnosticLevel::Info => log::info!(target: "texpio::diagnostic", "{text}"),
        DiagnosticLevel::Warn => log::warn!(target: "texpio::diagnostic", "{text}"),
        DiagnosticLevel::Error => log::error!(target: "texpio::diagnostic", "{text}"),
    }
}
