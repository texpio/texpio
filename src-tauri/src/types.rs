use serde::{Deserialize, Serialize};
use specta::Type;

pub const MAX_RECOVERY_DATA_BYTES: usize = 10_485_760;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Fr,
    Ar,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct AppPreferences {
    pub theme: Theme,
    pub quick_pane_shortcut: Option<String>,
    pub language: Option<Language>,
}

impl AppPreferences {
    pub fn validate(&self) -> Result<(), ServiceError> {
        if self.quick_pane_shortcut.as_deref().is_some_and(|value| {
            value.is_empty()
                || value.len() > 128
                || !value.is_ascii()
                || value.chars().any(char::is_control)
        }) {
            return Err(ServiceError::InvalidInput);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ServiceError {
    InvalidInput,
    CorruptData,
    Io,
    NotFound,
    DataTooLarge,
    Unavailable,
}

pub fn validate_filename(value: &str) -> Result<(), ServiceError> {
    let mut parts = value.split('.');
    let base = parts.next().unwrap_or("");
    let extension = parts.next();
    let reserved = base.to_ascii_uppercase();
    if value.len() > 100
        || base.is_empty()
        || !base
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        || extension
            .is_some_and(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_alphanumeric()))
        || parts.next().is_some()
        || matches!(reserved.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|prefix| {
            reserved.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
            })
        })
    {
        return Err(ServiceError::InvalidInput);
    }
    Ok(())
}
