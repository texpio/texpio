use crate::engine::{runtime::ProbeHandle, types::ProbePhase};
use std::sync::Arc;
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
};

struct ProbeTrayItems {
    pause: MenuItem<tauri::Wry>,
    stop: MenuItem<tauri::Wry>,
}

pub fn refresh_probe_tray(app: &AppHandle) {
    let (Some(items), Some(handle)) = (
        app.try_state::<ProbeTrayItems>(),
        app.try_state::<Arc<ProbeHandle>>(),
    ) else {
        return;
    };
    let phase = handle.status().phase;
    let _ = items.pause.set_text(if phase == ProbePhase::Paused {
        "Resume expansions"
    } else {
        "Pause expansions"
    });
    let _ = items
        .pause
        .set_enabled(matches!(phase, ProbePhase::Running | ProbePhase::Paused));
    let _ = items.stop.set_enabled(phase != ProbePhase::Stopped);
    if let Some(tray) = app.tray_by_id("texpio-tray") {
        let text = match phase {
            ProbePhase::Stopped => "Texpio — Stopped",
            ProbePhase::Running => "Texpio — Running",
            ProbePhase::Paused => "Texpio — Paused",
            ProbePhase::Fault => "Texpio — Fault",
        };
        let _ = tray.set_tooltip(Some(text));
    }
}

pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or(tauri::Error::WindowNotFound)?;
    window.unminimize()?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Open Texpio", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "probe-pause", "Pause expansions", false, None::<&str>)?;
    let stop = MenuItem::with_id(app, "probe-stop", "Stop expansions", false, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &pause, &stop, &quit])?;
    app.manage(ProbeTrayItems { pause, stop });
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("application icon".into()))?;

    TrayIconBuilder::with_id("texpio-tray")
        .icon(icon)
        .tooltip("Texpio — Stopped")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if show_main_window(app).is_err() {
                    eprintln!("Texpio could not restore its window");
                }
            }
            "probe-pause" => {
                if let Some(handle) = app.try_state::<Arc<ProbeHandle>>() {
                    let paused = handle.status().phase != ProbePhase::Paused;
                    let _ = handle.set_paused(paused);
                }
                refresh_probe_tray(app);
            }
            "probe-stop" => {
                if let Some(handle) = app.try_state::<Arc<ProbeHandle>>() {
                    handle.stop();
                }
                refresh_probe_tray(app);
            }
            "quit" => {
                crate::lifecycle::stop_then_exit(
                    || {
                        if let Some(handle) = app.try_state::<Arc<ProbeHandle>>() {
                            handle.stop();
                        }
                    },
                    || {
                        // Keep the app alive if the renderer is unavailable or its
                        // listener is not ready. Only main can acknowledge Quit.
                        let _ = show_main_window(app);
                        let _ = app.emit_to("main", "quit-requested", ());
                    },
                );
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Enter { .. } | TrayIconEvent::Click { .. }
            ) {
                refresh_probe_tray(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
