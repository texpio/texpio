//! Windows product engine. The stricter Developer probe remains separate.
#[cfg(target_os = "windows")]
pub mod clipboard;
pub mod commands;
#[cfg(target_os = "windows")]
pub mod desktop_native;
pub mod desktop_settings;
pub mod effects;
#[cfg(target_os = "windows")]
mod hooks;
pub mod matcher;
pub mod payload;
pub mod service;
pub mod settings;
#[cfg(target_os = "windows")]
mod startup;
pub mod store;
#[cfg(target_os = "windows")]
pub mod windows;
