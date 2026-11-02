//! Coarse platform readiness only. Native events, target identity and clipboard stay private.
use serde::Serialize;
use specta::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NativeBackend {
    Windows,
    Macos,
    LinuxX11,
    LinuxWayland,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NativeReadiness {
    Ready,
    PermissionRequired,
    UnsupportedSession,
    HelperRequired,
    BridgeRequired,
    FieldEvidenceRequired,
    Unavailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct NativePermissions {
    pub accessibility: bool,
    pub listen: bool,
    pub post: bool,
}
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProductCapabilities {
    pub backend: NativeBackend,
    pub readiness: NativeReadiness,
    pub permissions: NativePermissions,
    pub clipboard_shortcut: bool,
    pub autostart: bool,
    pub rich_text: bool,
    pub attachments: bool,
}
fn permissions() -> NativePermissions {
    #[cfg(target_os = "macos")]
    {
        let state = super::platform::macos::permission_state();
        NativePermissions {
            accessibility: state.accessibility,
            listen: state.listen,
            post: state.post,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        NativePermissions {
            accessibility: true,
            listen: true,
            post: true,
        }
    }
}
pub fn current() -> ProductCapabilities {
    #[cfg(target_os = "windows")]
    let (backend, readiness) = (NativeBackend::Windows, NativeReadiness::Ready);
    #[cfg(target_os = "macos")]
    let (backend, readiness) = {
        use super::platform::macos;
        (
            NativeBackend::Macos,
            if !macos::permission_state().ready() {
                NativeReadiness::PermissionRequired
            } else {
                match macos::readiness() {
                    Ok(()) => NativeReadiness::Ready,
                    Err(super::effects::ProductError::InputBlocked) => {
                        NativeReadiness::FieldEvidenceRequired
                    }
                    Err(_) => NativeReadiness::Unavailable,
                }
            },
        )
    };
    #[cfg(all(target_os = "linux", feature = "linux-x11"))]
    let (backend, readiness) = (
        NativeBackend::LinuxX11,
        if std::env::var("XDG_SESSION_TYPE").as_deref() != Ok("x11")
            || std::env::var_os("WAYLAND_DISPLAY").is_some()
        {
            NativeReadiness::UnsupportedSession
        } else {
            match super::platform::readiness() {
                Ok(()) => NativeReadiness::Ready,
                Err(super::effects::ProductError::InputBlocked) => {
                    NativeReadiness::FieldEvidenceRequired
                }
                Err(_) => NativeReadiness::Unavailable,
            }
        },
    );
    #[cfg(all(target_os = "linux", feature = "linux-wayland"))]
    let (backend, readiness) = (
        NativeBackend::LinuxWayland,
        super::platform::wayland_readiness(),
    );
    ProductCapabilities {
        backend,
        readiness,
        permissions: permissions(),
        clipboard_shortcut: cfg!(target_os = "windows"),
        autostart: true,
        rich_text: false,
        attachments: cfg!(target_os = "windows"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn platform_report_contains_no_native_input_or_context() {
        let value = serde_json::to_value(super::current()).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object.len(), 7);
        for name in ["clipboardShortcut", "autostart", "richText", "attachments"] {
            assert!(object[name].is_boolean());
        }
        assert!(object["backend"].is_string());
        assert!(object["readiness"].is_string());
        let permissions = object["permissions"].as_object().unwrap();
        assert_eq!(permissions.len(), 3);
        for name in ["accessibility", "listen", "post"] {
            assert!(permissions[name].is_boolean());
        }
    }
}
