fn main() {
    let msvc = std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let attributes = if msvc {
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())
    } else {
        tauri_build::Attributes::new()
    };
    tauri_build::try_build(
        attributes.app_manifest(tauri_build::AppManifest::new().commands(&[
            "get_app_info",
            "load_preferences",
            "save_preferences",
            "migrate_legacy_theme",
            "save_emergency_data",
            "load_emergency_data",
            "cleanup_old_recovery_files",
            "send_native_notification",
            "probe_status",
            "probe_fixtures",
            "probe_start",
            "probe_stop",
            "probe_pause",
            "probe_exclude",
            "show_quick_pane",
            "dismiss_quick_pane",
            "toggle_quick_pane",
            "get_default_quick_pane_shortcut",
            "update_quick_pane_shortcut",
        ])),
    )
    .expect("could not prepare Tauri build");
    if msvc {
        // Tests and binding-export examples also link Tauri's native dialogs.
        // They need Common Controls v6, just like the packaged application.
        println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo::rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}
