//! Native floating entry window. It never inserts text in another application.
use crate::{
    commands::preferences::PreferencesStore,
    shortcuts::{DEFAULT_SHORTCUT, ShortcutActor, ShortcutBackend},
    types::ServiceError,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tauri::{AppHandle, Manager, State, WebviewUrl};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
const LABEL: &str = "quick-pane";
#[cfg(target_os = "macos")]
use tauri_nspanel::{
    CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask, tauri_panel,
};
#[cfg(target_os = "macos")]
tauri_panel! { panel!(QuickPanePanel { config: { can_become_key_window: true, can_become_main_window: false, is_floating_panel: true } }) }

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    {
        use tauri::{LogicalSize, Size};
        let panel = PanelBuilder::<_, QuickPanePanel>::new(app, LABEL)
            .url(WebviewUrl::App("quick-pane.html".into()))
            .title("Texpio Quick Entry")
            .size(Size::Logical(LogicalSize::new(500.0, 72.0)))
            .level(PanelLevel::Status)
            .transparent(true)
            .has_shadow(true)
            .collection_behavior(
                CollectionBehavior::new()
                    .full_screen_auxiliary()
                    .can_join_all_spaces(),
            )
            .style_mask(StyleMask::empty().nonactivating_panel())
            .hides_on_deactivate(false)
            .works_when_modal(true)
            .with_window(|window| {
                window
                    .decorations(false)
                    .transparent(true)
                    .skip_taskbar(true)
                    .resizable(false)
                    .center()
            })
            .build()?;
        panel.hide();
    }
    #[cfg(not(target_os = "macos"))]
    tauri::WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("quick-pane.html".into()))
        .title("Texpio Quick Entry")
        .inner_size(500.0, 72.0)
        .always_on_top(true)
        .skip_taskbar(true)
        .decorations(false)
        .transparent(true)
        .visible(false)
        .resizable(false)
        .center()
        .build()?;
    Ok(())
}
fn position(app: &AppHandle) {
    let monitor = app
        .cursor_position()
        .ok()
        .and_then(|cursor| app.monitor_from_point(cursor.x, cursor.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let (Some(monitor), Some(window)) = (monitor, app.get_webview_window(LABEL)) {
        let size = monitor.size();
        let origin = monitor.position();
        let scale = monitor.scale_factor();
        let x = origin.x as f64 + (size.width as f64 - 500.0 * scale) / 2.0;
        let y = origin.y as f64 + (size.height as f64 - 72.0 * scale) / 2.0;
        let _ = window.set_position(tauri::PhysicalPosition::new(x as i32, y as i32));
    }
}
#[tauri::command]
#[specta::specta]
pub fn show_quick_pane(app: AppHandle) -> Result<(), ServiceError> {
    position(&app);
    #[cfg(target_os = "macos")]
    {
        app.get_webview_panel(LABEL)
            .map_err(|_| ServiceError::Unavailable)?
            .show_and_make_key();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let window = app
            .get_webview_window(LABEL)
            .ok_or(ServiceError::Unavailable)?;
        window
            .show()
            .and_then(|()| window.set_focus())
            .map_err(|_| ServiceError::Unavailable)?;
    }
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub fn dismiss_quick_pane(app: AppHandle) -> Result<(), ServiceError> {
    #[cfg(target_os = "macos")]
    {
        let panel = app
            .get_webview_panel(LABEL)
            .map_err(|_| ServiceError::Unavailable)?;
        if panel.is_visible() {
            panel.resign_key_window();
            panel.hide();
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        app.get_webview_window(LABEL)
            .ok_or(ServiceError::Unavailable)?
            .hide()
            .map_err(|_| ServiceError::Unavailable)?;
    }
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub fn toggle_quick_pane(app: AppHandle) -> Result<(), ServiceError> {
    #[cfg(target_os = "macos")]
    let visible = app
        .get_webview_panel(LABEL)
        .map_err(|_| ServiceError::Unavailable)?
        .is_visible();
    #[cfg(not(target_os = "macos"))]
    let visible = app
        .get_webview_window(LABEL)
        .ok_or(ServiceError::Unavailable)?
        .is_visible()
        .map_err(|_| ServiceError::Unavailable)?;
    if visible {
        dismiss_quick_pane(app)
    } else {
        show_quick_pane(app)
    }
}
#[tauri::command]
#[specta::specta]
pub fn get_default_quick_pane_shortcut() -> String {
    DEFAULT_SHORTCUT.into()
}
#[tauri::command]
#[specta::specta]
pub async fn update_quick_pane_shortcut(
    actor: State<'_, ShortcutActor>,
    shortcut: String,
) -> Result<(), ServiceError> {
    let actor = actor.inner().clone();
    crate::utils::storage::blocking(move || actor.replace(shortcut)).await
}
struct NativeBackend {
    app: AppHandle,
    preferences: Arc<PreferencesStore>,
    active: Arc<AtomicU64>,
}
impl ShortcutBackend for NativeBackend {
    fn canonicalize(&self, chord: &str) -> Result<String, ServiceError> {
        if chord.len() > 128
            || !chord.is_ascii()
            || !chord.split('+').any(|part| {
                matches!(
                    part.to_ascii_lowercase().as_str(),
                    "commandorcontrol"
                        | "cmdorctrl"
                        | "ctrl"
                        | "control"
                        | "cmd"
                        | "command"
                        | "alt"
                        | "super"
                        | "meta"
                )
            })
        {
            return Err(ServiceError::InvalidInput);
        }
        chord
            .parse::<Shortcut>()
            .map(|value| value.to_string())
            .map_err(|_| ServiceError::InvalidInput)
    }
    fn register(&mut self, chord: &str, generation: u64) -> Result<(), ServiceError> {
        let active = self.active.clone();
        self.app
            .global_shortcut()
            .on_shortcut(chord, move |app, _, event| {
                if event.state == ShortcutState::Pressed
                    && active.load(Ordering::Acquire) == generation
                {
                    let _ = toggle_quick_pane(app.clone());
                }
            })
            .map_err(|_| ServiceError::Unavailable)
    }
    fn unregister(&mut self, chord: &str) -> Result<(), ServiceError> {
        self.app
            .global_shortcut()
            .unregister(chord)
            .map_err(|_| ServiceError::Unavailable)
    }
    fn load(&self) -> Result<Option<String>, ServiceError> {
        Ok(self.preferences.load()?.quick_pane_shortcut)
    }
    fn save(&mut self, chord: Option<String>) -> Result<(), ServiceError> {
        self.preferences.save_shortcut(chord)
    }
}
pub fn start_shortcut_worker(app: &AppHandle, preferences: Arc<PreferencesStore>) {
    let active = Arc::new(AtomicU64::new(0));
    app.manage(ShortcutActor::spawn(
        NativeBackend {
            app: app.clone(),
            preferences,
            active: active.clone(),
        },
        active,
    ));
}
