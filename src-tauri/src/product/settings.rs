use super::matcher::{Activation, Boundary};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum InsertionMode {
    #[default]
    Paste,
    Type,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TypingSpeed {
    Fast,
    #[default]
    Balanced,
    Careful,
}
impl TypingSpeed {
    pub fn events_per_second(self) -> u32 {
        match self {
            Self::Fast => 500,
            Self::Balanced => 300,
            Self::Careful => 120,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ProductSettings {
    // Legacy file/IPC preference. Fresh process startup always attempts expansion.
    pub enabled: bool,
    pub boundary: Boundary,
    pub activation: Activation,
    pub preserve_terminator: bool,
    pub insertion: InsertionMode,
    pub restore_clipboard: bool,
    pub typing_speed: TypingSpeed,
    pub cancel_key: String,
    pub clipboard_enabled: bool,
    pub clipboard_shortcut: String,
    pub clipboard_mode: InsertionMode,
    pub allow_software_input: bool,
    pub toggle_shortcut: String,
    pub autostart: bool,
    pub close_to_tray: bool,
    pub attachment_delay_ms: u32,
}
impl Default for ProductSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            boundary: Boundary::Word,
            activation: Activation::Immediate,
            preserve_terminator: true,
            insertion: InsertionMode::Paste,
            restore_clipboard: true,
            typing_speed: TypingSpeed::Balanced,
            cancel_key: "Escape".into(),
            clipboard_enabled: true,
            clipboard_shortcut: "CmdOrCtrl+Shift+V".into(),
            clipboard_mode: InsertionMode::Paste,
            allow_software_input: false,
            toggle_shortcut: "CmdOrCtrl+Alt+Shift+Space".into(),
            autostart: false,
            close_to_tray: true,
            attachment_delay_ms: 500,
        }
    }
}
impl ProductSettings {
    pub fn validate_for_platform(&self, platform: &str) -> bool {
        self.validate()
            && (platform != "macos" || !matches!(self.cancel_key.as_str(), "Pause" | "ScrollLock"))
    }
    pub fn validate(&self) -> bool {
        let shortcut = |value: &str| {
            !value.trim().is_empty()
                && value.len() <= 128
                && value.is_ascii()
                && !value.chars().any(char::is_control)
        };
        self.cancel_vk().is_some()
            && (100..=5_000).contains(&self.attachment_delay_ms)
            && shortcut(&self.clipboard_shortcut)
            && shortcut(&self.toggle_shortcut)
            && (!self.clipboard_enabled || self.clipboard_shortcut != self.toggle_shortcut)
    }
    pub fn cancel_vk(&self) -> Option<u16> {
        match self.cancel_key.as_str() {
            "Escape" => Some(0x1B),
            "Pause" => Some(0x13),
            "ScrollLock" => Some(0x91),
            value => value
                .strip_prefix('F')
                .and_then(|n| n.parse::<u16>().ok())
                .filter(|n| (1..=12).contains(n))
                .map(|n| 0x70 + n - 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn macos_rejects_unavailable_cancel_keys_before_persisting_settings() {
        for key in ["Pause", "ScrollLock"] {
            let settings = ProductSettings {
                cancel_key: key.into(),
                ..Default::default()
            };
            assert!(settings.validate_for_platform("windows"));
            assert!(!settings.validate_for_platform("macos"));
        }
        for key in ["Escape", "F1", "F12"] {
            assert!(
                ProductSettings {
                    cancel_key: key.into(),
                    ..Default::default()
                }
                .validate_for_platform("macos")
            );
        }
    }
    #[test]
    fn defaults_request_startup_with_explicit_paste_and_safe_cancel() {
        let settings = ProductSettings::default();
        assert!(settings.enabled);
        assert!(!settings.autostart);
        assert!(!settings.allow_software_input);
        assert_eq!(settings.activation, Activation::Immediate);
        assert_eq!(settings.insertion, InsertionMode::Paste);
        assert_eq!(settings.clipboard_mode, InsertionMode::Paste);
        assert_eq!(settings.cancel_vk(), Some(0x1B));
        assert_eq!(settings.typing_speed.events_per_second(), 300);
        assert_eq!(
            serde_json::from_str::<ProductSettings>("{}").unwrap(),
            settings
        );
    }
    #[test]
    fn activation_round_trips_and_legacy_settings_become_immediate() {
        let old: ProductSettings =
            serde_json::from_str(r#"{"preserveTerminator":false,"allowSoftwareInput":true}"#)
                .unwrap();
        assert_eq!(old.activation, Activation::Immediate);
        assert!(!old.preserve_terminator);
        assert!(old.allow_software_input);
        for activation in [Activation::Immediate, Activation::Terminator] {
            let settings = ProductSettings {
                activation,
                ..old.clone()
            };
            assert_eq!(
                serde_json::from_str::<ProductSettings>(&serde_json::to_string(&settings).unwrap())
                    .unwrap(),
                settings
            );
        }
        assert!(serde_json::from_str::<ProductSettings>(r#"{"activation":"unknown"}"#).is_err());
    }
    #[test]
    fn invalid_shortcuts_delays_and_cancel_keys_are_refused() {
        let mut settings = ProductSettings {
            attachment_delay_ms: 0,
            ..Default::default()
        };
        assert!(!settings.validate());
        settings.attachment_delay_ms = 500;
        settings.cancel_key = "F13".into();
        assert!(!settings.validate());
        settings.cancel_key = "F2".into();
        assert!(settings.validate());
        settings.clipboard_shortcut = "bad\nshortcut".into();
        assert!(!settings.validate());
    }
}
