mod commands;
#[cfg(desktop)]
mod desktop;
pub mod engine;
mod probe_commands;

#[cfg(desktop)]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if desktop::show_main_window(app).is_err() {
                eprintln!("Texpio could not restore its window");
            }
        }))
        .setup(|app| {
            desktop::setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                if window.hide().is_err() {
                    window.app_handle().exit(1);
                }
            }
        });

    builder
        .manage(std::sync::Arc::new(engine::runtime::ProbeHandle::new()))
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            probe_commands::probe_status,
            probe_commands::probe_fixtures,
            probe_commands::probe_start,
            probe_commands::probe_stop,
            probe_commands::probe_pause,
            probe_commands::probe_exclude
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) && let Some(handle) =
                app.try_state::<std::sync::Arc<engine::runtime::ProbeHandle>>()
            {
                handle.stop();
            }
        });
}
