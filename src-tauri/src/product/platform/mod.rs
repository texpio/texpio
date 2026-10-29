//! Compile-time input backend selection. Linux distributions are separate products.
#[cfg(target_os = "windows")]
pub(crate) use super::hooks::HookRuntime as Runtime;
#[cfg(target_os = "macos")]
pub(crate) mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::Runtime;
#[cfg(all(target_os = "linux", feature = "linux-x11"))]
mod linux_x11;
#[cfg(all(target_os = "linux", feature = "linux-x11"))]
pub(crate) use linux_x11::Runtime;
#[cfg(all(target_os = "linux", feature = "linux-x11"))]
pub(crate) use linux_x11::readiness;
#[cfg(all(target_os = "linux", feature = "linux-wayland"))]
mod linux_wayland;
#[cfg(all(target_os = "linux", feature = "linux-wayland"))]
pub(crate) use linux_wayland::Runtime;

#[cfg(all(target_os = "linux", feature = "linux-wayland"))]
pub(crate) fn wayland_readiness() -> super::capabilities::NativeReadiness {
    use super::capabilities::NativeReadiness as State;
    use linux_wayland::Readiness;
    match linux_wayland::readiness() {
        Readiness::Ready => State::Ready,
        Readiness::HelperMissing => State::HelperRequired,
        Readiness::BridgeMissing => State::BridgeRequired,
        Readiness::BridgeAuthenticationUnavailable => State::Unavailable,
        Readiness::SessionUnavailable => State::UnsupportedSession,
        Readiness::FieldEvidenceUnavailable => State::FieldEvidenceRequired,
    }
}

#[cfg(all(test, not(target_os = "macos")))]
#[path = "macos/policy.rs"]
#[allow(dead_code)]
mod macos_policy_tests;
#[cfg(all(test, not(all(target_os = "linux", feature = "linux-x11"))))]
#[path = "linux_x11/policy.rs"]
#[allow(dead_code)]
mod x11_policy_tests;

#[cfg(all(target_os = "linux", feature = "linux-x11", feature = "linux-wayland"))]
compile_error!("Select one Linux variant: linux-x11 or linux-wayland, never both");
#[cfg(all(
    target_os = "linux",
    not(any(feature = "linux-x11", feature = "linux-wayland"))
))]
compile_error!("Select a Linux variant: linux-x11 or linux-wayland");
