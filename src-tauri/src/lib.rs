mod commands;
#[cfg(desktop)]
mod desktop;

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
        .invoke_handler(tauri::generate_handler![commands::get_app_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
