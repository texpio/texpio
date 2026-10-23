pub mod bindings;
mod commands;
#[cfg(desktop)]
mod desktop;
pub mod engine;
mod lifecycle;
#[cfg(test)]
mod lifecycle_service_tests;
mod probe_commands;
#[cfg(test)]
mod service_tests;
mod shortcuts;
mod types;
#[cfg(test)]
mod lifecycle_tests {
    use super::lifecycle::{CloseBehavior, close_behavior, stop_then_exit};
    #[test]
    fn engine_stop_precedes_exit_and_hiding_never_starts_workers() {
        let calls = std::cell::RefCell::new(vec![]);
        stop_then_exit(
            || calls.borrow_mut().push("stop"),
            || calls.borrow_mut().push("exit"),
        );
        assert_eq!(*calls.borrow(), vec!["stop", "exit"]);
        assert_eq!(close_behavior("main"), CloseBehavior::Hide);
        assert_eq!(close_behavior("quick-pane"), CloseBehavior::Hide);
        assert_eq!(close_behavior("other"), CloseBehavior::Default);
    }
}
#[cfg(test)]
mod shortcut_tests;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();
    let configuration = commands::lifecycle::UpdateConfiguration {
        configured: context
            .config()
            .plugins
            .0
            .get("updater")
            .is_some_and(commands::lifecycle::valid_update_configuration),
    };
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if desktop::show_main_window(app).is_err() {
                eprintln!("Texpio could not restore its window");
            }
        }))
        .on_window_event(|window, event| {
            if lifecycle::close_behavior(window.label()) == lifecycle::CloseBehavior::Hide
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let result = if window.label() == "quick-pane" {
                    commands::quick_pane::dismiss_quick_pane(window.app_handle().clone())
                        .map_err(|_| ())
                } else {
                    window.hide().map_err(|_| ())
                };
                if result.is_err() {
                    window.app_handle().exit(1);
                }
            }
        });

    #[cfg(desktop)]
    let builder = builder
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED
                        | tauri_plugin_window_state::StateFlags::FULLSCREEN,
                )
                .with_denylist(&["quick-pane"])
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build());
    #[cfg(target_os = "macos")]
    let builder = builder.plugin(tauri_nspanel::init());

    // Missing or invalid configuration must not initialize the updater plugin.
    #[cfg(desktop)]
    let builder = if configuration.configured {
        builder.plugin(tauri_plugin_updater::Builder::new().build())
    } else {
        builder
    };

    builder
        .manage(configuration)
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .filter(|metadata| metadata.target() == "texpio::diagnostic")
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("texpio".into()),
                    }),
                ])
                .max_file_size(1_048_576)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            let preferences =
                std::sync::Arc::new(commands::preferences::PreferencesStore::new(root.clone()));
            app.manage(preferences.clone());
            app.manage(std::sync::Arc::new(commands::recovery::RecoveryStore::new(
                root.join("recovery"),
            )));
            #[cfg(desktop)]
            {
                desktop::setup_tray(app.handle())?;
                // Window failure and shortcut conflict cannot prevent the main app from starting.
                let _ = commands::quick_pane::init(app.handle());
                commands::quick_pane::start_shortcut_worker(app.handle(), preferences);
            }
            Ok(())
        })
        .manage(std::sync::Arc::new(engine::runtime::ProbeHandle::new()))
        .invoke_handler(bindings::builder().invoke_handler())
        .build(context)
        .unwrap_or_else(|_| {
            eprintln!("Texpio could not start");
            std::process::exit(1)
        })
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if matches!(event, tauri::RunEvent::Reopen { .. }) {
                let _ = desktop::show_main_window(app);
            }
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
