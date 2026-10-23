pub mod bindings;
mod commands;
#[cfg(desktop)]
mod desktop;
pub mod engine;
mod probe_commands;
#[cfg(test)]
mod service_tests;
mod types;
mod utils;

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
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            app.manage(std::sync::Arc::new(
                commands::preferences::PreferencesStore::new(root.clone()),
            ));
            app.manage(std::sync::Arc::new(commands::recovery::RecoveryStore::new(
                root.join("recovery"),
            )));
            #[cfg(desktop)]
            desktop::setup_tray(app.handle())?;
            Ok(())
        })
        .manage(std::sync::Arc::new(engine::runtime::ProbeHandle::new()))
        .invoke_handler(bindings::builder().invoke_handler())
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
