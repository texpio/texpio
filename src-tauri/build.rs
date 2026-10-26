fn main() {
    // Recompile native icon resources when branding changes in an incremental build.
    println!("cargo::rerun-if-changed=icons");
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
            "library_load",
            "library_save",
            "library_export",
            "library_import_preview",
            "library_import",
            "library_start",
            "library_metadata",
            "library_set_favorite",
            "attachment_list",
            "attachment_settings",
            "library_location",
            "library_switch",
            "backup_export",
            "backup_preview",
            "backup_discard",
            "backup_import",
            "library_recovery_status",
            "library_recover",
            "attachment_add",
            "attachment_add_data",
            "attachment_remove",
            "attachment_reorder",
            "attachment_save_settings",
            "product_status",
            "product_start",
            "product_stop",
            "product_pause",
            "product_settings",
            "product_save_settings",
            "show_quick_pane",
            "dismiss_quick_pane",
            "toggle_quick_pane",
            "get_default_quick_pane_shortcut",
            "update_quick_pane_shortcut",
            "get_update_configuration",
            "prepare_for_relaunch",
            "quit_app",
            "record_diagnostic",
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
