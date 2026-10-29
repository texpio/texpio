//! Native product engine. The stricter Windows Developer probe remains separate.
pub mod capabilities;
#[cfg(target_os = "windows")]
pub mod clipboard;
pub mod commands;
pub mod desktop_native;
pub mod desktop_settings;
pub mod effects;
#[cfg(target_os = "windows")]
mod hooks;
pub mod matcher;
pub mod payload;
pub(crate) mod platform;
#[cfg(any(target_os = "windows", test))]
mod rich_clipboard;
pub mod service;
pub mod settings;
#[cfg(target_os = "windows")]
mod startup;
#[cfg(not(target_os = "windows"))]
#[path = "startup_unix.rs"]
mod startup;
#[cfg(all(test, target_os = "windows"))]
#[path = "startup_unix.rs"]
#[allow(dead_code)]
mod startup_unix_tests;
pub mod store;
pub(crate) mod tray_action;
#[cfg(target_os = "windows")]
pub mod windows;
