use crate::{
    commands::lifecycle::{confirm_stopped, valid_update_configuration},
    engine::types::{ProbePhase, RefusalReason},
};
use serde_json::json;
#[test]
fn linux_update_feeds_never_cross_desktop_variants() {
    use crate::commands::lifecycle::valid_update_variant;
    let x11 = json!({"endpoints":["https://github.com/texpio/texpio/releases/latest/download/latest-linux-x11.json"]});
    let wayland = json!({"endpoints":["https://github.com/texpio/texpio/releases/latest/download/latest-linux-wayland.json"]});
    assert!(valid_update_variant(&x11, Some("x11")));
    assert!(valid_update_variant(&wayland, Some("wayland")));
    assert!(!valid_update_variant(&x11, Some("wayland")));
    assert!(!valid_update_variant(&wayland, Some("x11")));
    assert!(!valid_update_variant(
        &json!({"endpoints":["https://github.com/latest.json"]}),
        Some("x11")
    ));
    assert!(!valid_update_variant(
        &json!({"endpoints":[]}),
        Some("wayland")
    ));
    assert!(!valid_update_variant(&json!({}), Some("wayland")));
}
#[test]
fn unconfigured_or_placeholder_feeds_are_refused() {
    for value in [
        json!(null),
        json!({}),
        json!({"pubkey":"YOUR_PUBLIC_KEY","endpoints":["https://YOUR_FEED/latest.json"]}),
        json!({"pubkey":"not-a-key","endpoints":["http://updates.texpio.test/latest.json"]}),
        json!({"pubkey":"not-a-key","endpoints":["https://user:password@updates.texpio.test/latest.json"]}),
    ] {
        assert!(!valid_update_configuration(&value));
    }
}
#[test]
fn relaunch_requires_confirmed_stopped_including_fault_or_timeout() {
    assert_eq!(confirm_stopped(ProbePhase::Stopped), Ok(()));
    for phase in [ProbePhase::Running, ProbePhase::Paused, ProbePhase::Fault] {
        assert_eq!(confirm_stopped(phase), Err(RefusalReason::OutcomeUncertain));
    }
}
#[test]
fn configured_updates_require_a_real_parseable_public_key_and_https_feed() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    // Existing public fixture, no keys generated: official updater2.14.0 test at
    // https://github.com/tauri-apps/plugins-workspace/blob/44293a5722efcaf27060504e63b17aeba721b40f/plugins/updater/src/updater.rs#L2117
    let public_key = STANDARD.encode("untrusted comment: minisign public key: A49EF4746FA6E6D7\nRWTX5qZvdPSepDGWg2bz8n6ArQnBMwgQK1geVosorcbLzcunD+KJ+cYe");
    let feed = "https://github.com/texpio/texpio/releases/latest/download/latest.json";
    assert!(valid_update_configuration(
        &json!({"pubkey":public_key,"endpoints":[feed]})
    ));
    for invalid in [
        "http://github.com/latest.json",
        "https://user:password@github.com/latest.json",
        "https://example.com/latest.json",
        "https://github.com/YOUR_REPO/latest.json",
        "not a URL",
    ] {
        assert!(!valid_update_configuration(
            &json!({"pubkey":public_key,"endpoints":[invalid]})
        ));
    }
}
#[test]
fn diagnostic_command_rejects_unknown_events_and_native_input_fields() {
    use crate::commands::lifecycle::DiagnosticRecord;
    assert!(
        serde_json::from_value::<DiagnosticRecord>(
            json!({"level":"error","event":"renderFailed","count":1})
        )
        .is_ok()
    );
    assert!(serde_json::from_value::<DiagnosticRecord>(json!({"level":"error","event":"renderFailed","count":1,"HWND":12345,"clipboard":"secret"})).is_err());
    assert!(
        serde_json::from_value::<DiagnosticRecord>(
            json!({"level":"error","event":"C:/private/secret","count":null})
        )
        .is_err()
    );
}
