//! Windows product engine. The stricter Developer probe remains separate.
#[cfg(target_os = "windows")]
pub mod clipboard;
pub mod effects;
#[cfg(target_os = "windows")]
mod hooks;
pub mod matcher;
pub mod service;
pub mod settings;
#[cfg(target_os = "windows")]
pub mod windows;
