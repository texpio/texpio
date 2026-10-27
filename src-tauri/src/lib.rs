pub mod bindings;
mod commands;
#[cfg(test)]
mod coordination_tests;
#[cfg(desktop)]
mod desktop;
pub mod engine;
pub mod library;
mod library_commands;
mod lifecycle;
#[cfg(test)]
mod lifecycle_service_tests;
mod probe_commands;
pub mod product;
mod rich_text;
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
        assert_eq!(close_behavior("main", true), CloseBehavior::Hide);
        assert_eq!(close_behavior("quick-pane", true), CloseBehavior::Hide);
        assert_eq!(close_behavior("other", true), CloseBehavior::Default);
    }
    #[test]
    fn startup_and_close_options_are_explicit_without_starting_hooks() {
        assert_eq!(close_behavior("main", false), CloseBehavior::Quit);
        assert_eq!(close_behavior("quick-pane", false), CloseBehavior::Hide);
        assert!(!super::lifecycle::hidden_startup(["file--hidden".into()]));
        assert!(super::lifecycle::hidden_startup(["--hidden".into()]));
        assert_eq!(
            super::lifecycle::startup_command(std::path::Path::new(
                "C:\\Program Files\\Texpio\\texpio.exe"
            ))
            .unwrap(),
            "\"C:\\Program Files\\Texpio\\texpio.exe\" --hidden"
        );
    }
}
#[cfg(test)]
mod shortcut_tests;
mod utils;
mod window_layout;

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
    let builder = tauri::Builder::default().manage(window_layout::InitialWindowLayout::default());

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if desktop::show_main_window(app).is_err() {
                eprintln!("Texpio could not restore its window");
            }
        }))
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let close_to_tray = window
                    .app_handle()
                    .try_state::<std::sync::Arc<product::store::SettingsStore>>()
                    .and_then(|store| store.load().ok())
                    .is_none_or(|settings| settings.close_to_tray);
                let behavior = lifecycle::close_behavior(window.label(), close_to_tray);
                if behavior == lifecycle::CloseBehavior::Default {
                    return;
                }
                api.prevent_close();
                if behavior == lifecycle::CloseBehavior::Quit {
                    desktop::request_quit(window.app_handle());
                    return;
                }
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
                .skip_initial_state("main")
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
            app.manage(std::sync::Arc::new(product::store::SettingsStore::new(
                root.clone(),
            )));
            let library = std::sync::Arc::new(library_commands::LibraryService::new(root.clone()));
            app.manage(library.clone());
            library_commands::location::start_watch(app.handle(), &library);
            app.manage(std::sync::Arc::new(commands::recovery::RecoveryStore::new(
                root.join("recovery"),
            )));
            #[cfg(desktop)]
            {
                desktop::setup_tray(app.handle())?;
                // Window failure and shortcut conflict cannot prevent the main app from starting.
                let _ = commands::quick_pane::init(app.handle());
                commands::quick_pane::start_shortcut_worker(app.handle(), preferences);
                #[cfg(target_os = "windows")]
                app.manage(product::desktop_native::DesktopSettingsActor::start(
                    app.handle(),
                ));
                if !lifecycle::hidden_startup(std::env::args_os().skip(1)) {
                    desktop::show_main_window(app.handle())?;
                }
            }
            Ok(())
        })
        .manage(std::sync::Arc::new(engine::runtime::ProbeHandle::new()))
        .manage(std::sync::Arc::new(product::service::ProductEngine::new()))
        .invoke_handler(bindings::builder().invoke_handler())
        .build(context)
        .unwrap_or_else(|_| {
            eprintln!("Texpio could not start");
            std::process::exit(1)
        })
        .run(|app, event| {
            #[cfg(target_os = "windows")]
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) && let Some(desktop) =
                app.try_state::<product::desktop_native::DesktopSettingsActor>()
            {
                desktop.shutdown();
            }
            if matches!(event, tauri::RunEvent::Exit)
                && let Some(library) =
                    app.try_state::<std::sync::Arc<library_commands::LibraryService>>()
            {
                library_commands::location::stop_watch(&library);
            }
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) && let Some(engine) =
                app.try_state::<std::sync::Arc<product::service::ProductEngine>>()
            {
                engine.stop();
            }
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
