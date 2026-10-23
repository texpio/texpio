use crate::types::ServiceError;
use tauri::AppHandle;

#[tauri::command]
#[specta::specta]
pub async fn send_native_notification(
    app: AppHandle,
    title: String,
    body: Option<String>,
) -> Result<(), ServiceError> {
    if title.trim().is_empty()
        || title.chars().count() > 128
        || title.contains('\0')
        || body
            .as_deref()
            .is_some_and(|value| value.chars().count() > 4096 || value.contains('\0'))
    {
        return Err(ServiceError::InvalidInput);
    }
    use tauri_plugin_notification::NotificationExt;
    let mut notification = app.notification().builder().title(title);
    if let Some(body) = body {
        notification = notification.body(body);
    }
    notification.show().map_err(|_| ServiceError::Unavailable)
}
