//! Pure policy shared by macOS capture and effects; runnable without native input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PermissionState {
    pub accessibility: bool,
    pub listen: bool,
    pub post: bool,
}
impl PermissionState {
    /// Accessibility trust is the grant macOS uses for input control: a trusted
    /// process may create an event tap and post events, and the listen/post
    /// preflights report true without a separate Input Monitoring entry. That
    /// entry exists for processes that are not trusted for Accessibility, so it
    /// is reported for diagnosis but must not gate readiness.
    pub fn ready(self) -> bool {
        self.accessibility
    }
}
pub(crate) fn context_valid(
    active: bool,
    generation: u64,
    expected: u64,
    same_target: bool,
    secure: bool,
) -> bool {
    active && generation == expected && same_target && !secure
}
pub(crate) fn lease_matches(current: isize, owned: isize, marker_matches: bool) -> bool {
    current == owned && marker_matches
}
pub(crate) fn unicode_events(text: &str) -> Vec<Vec<u16>> {
    text.chars()
        .map(|ch| ch.encode_utf16(&mut [0; 2]).to_vec())
        .collect()
}
pub(crate) fn payload_supported(text: &str, multiline: bool) -> bool {
    text.len() <= 1024 * 1024
        && (multiline || !text.contains('\n'))
        && !text.chars().any(|ch| ch.is_control() && ch != '\n')
}
pub(crate) fn mode_supported(paste: bool, restore: bool) -> bool {
    !paste || !restore
}
/// Keyboard layouts can reproduce the ASCII triggers this profile accepts.
/// Input methods (Pinyin, Kotoeri, transliteration) are excluded: they compose
/// marked text that a keydown cannot describe, so per-key deletion is unsafe.
pub(crate) fn layout_supported(is_keyboard_layout: bool, ascii_capable: bool) -> bool {
    is_keyboard_layout && ascii_capable
}
/// One keydown contributes one history character only when it delivered exactly
/// one printable ASCII scalar. Composition, dead-key output and control scalars
/// break the one-key/one-character assumption this profile deletes by.
pub(crate) fn accepted_scalar(text: &str) -> Option<char> {
    let mut scalars = text.chars();
    let character = scalars.next()?;
    (scalars.next().is_none() && character.is_ascii() && !character.is_control())
        .then_some(character)
}
/// The field must positively report an editable text role, must not report the
/// secure-text protection subrole, and must explicitly return empty marked text.
pub(crate) fn context_evidence_safe(
    role: &str,
    subrole: Option<&str>,
    marked: Option<&str>,
) -> bool {
    matches!(role, "AXTextField" | "AXTextArea")
        && subrole != Some("AXSecureTextField")
        && marked == Some("")
}
pub(crate) fn admission_supported(text: &str, multiline: bool, paste: bool, restore: bool) -> bool {
    mode_supported(paste, restore) && payload_supported(text, multiline)
}
pub(crate) fn admit<T>(
    busy: &std::sync::atomic::AtomicBool,
    jobs: &std::sync::mpsc::SyncSender<T>,
    job: T,
) -> bool {
    use std::sync::atomic::Ordering;
    if busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return false;
    }
    if jobs.try_send(job).is_ok() {
        true
    } else {
        busy.store(false, Ordering::SeqCst);
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paste_restoration_refuses_before_capture_or_job_admission() {
        assert!(!mode_supported(true, true));
        assert!(mode_supported(true, false));
        assert!(mode_supported(false, true));
        assert!(mode_supported(false, false));
    }
    #[test]
    fn only_editable_unprotected_fields_with_idle_composition_admit_context() {
        assert!(context_evidence_safe("AXTextField", None, Some("")));
        assert!(context_evidence_safe("AXTextArea", None, Some("")));
        assert!(context_evidence_safe(
            "AXTextField",
            Some("AXSearchField"),
            Some("")
        ));
        assert!(context_evidence_safe("AXTextField", Some(""), Some("")));
        assert!(context_evidence_safe(
            "AXTextArea",
            Some("AXUnknown"),
            Some("")
        ));
        for (role, subrole, marked) in [
            ("AXTextField", None, None),
            ("AXTextArea", None, None),
            ("AXTextField", Some("AXSearchField"), None),
            ("AXTextField", Some("AXSearchField"), Some("composing")),
            ("AXTextField", Some("AXSecureTextField"), Some("")),
            ("AXTextArea", Some("AXSecureTextField"), Some("")),
            ("AXStaticText", None, Some("")),
            ("AXButton", None, Some("")),
            ("", None, Some("")),
        ] {
            assert!(
                !context_evidence_safe(role, subrole, marked),
                "{role}/{subrole:?}/{marked:?} must be refused"
            );
        }
    }
    #[test]
    fn only_ascii_capable_keyboard_layouts_admit_capture() {
        assert!(layout_supported(true, true));
        assert!(!layout_supported(true, false));
        assert!(!layout_supported(false, true));
        assert!(!layout_supported(false, false));
    }
    #[test]
    fn only_single_printable_ascii_scalars_extend_history() {
        assert_eq!(accepted_scalar("a"), Some('a'));
        assert_eq!(accepted_scalar("~"), Some('~'));
        for text in ["", "ab", "é", "ß", "\u{301}", "\n", "\t", "\0", "👋"] {
            assert_eq!(accepted_scalar(text), None, "{text:?} must be refused");
        }
    }
    #[test]
    fn deterministic_refusal_never_consumes_queue_or_completing_event() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            mpsc,
        };
        for (text, multiline, paste, restore) in [
            ("one\ntwo", false, false, false),
            ("plain", false, true, true),
        ] {
            let busy = AtomicBool::new(false);
            let (sender, receiver) = mpsc::sync_channel(1);
            let suppress =
                admission_supported(text, multiline, paste, restore) && admit(&busy, &sender, 17);
            assert!(!suppress, "the original completing event must be delivered");
            assert!(!busy.load(Ordering::SeqCst));
            assert!(receiver.try_recv().is_err());
        }
        assert!(admission_supported("one\ntwo", true, false, false));
        assert!(admission_supported("plain", false, true, false));
    }
    #[test]
    fn controls_and_single_line_newlines_refuse_before_erasure() {
        assert!(payload_supported("Café 👋", false));
        assert!(payload_supported("one\ntwo", true));
        assert!(!payload_supported("one\ntwo", false));
        for text in ["a\tb", "a\rb", "a\0b"] {
            assert!(!payload_supported(text, true));
        }
        assert!(!payload_supported(&"x".repeat(1024 * 1024 + 1), true));
    }
    #[test]
    fn suppresses_only_the_exact_successfully_admitted_job() {
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            mpsc,
        };
        let busy = AtomicBool::new(false);
        let (sender, receiver) = mpsc::sync_channel(1);
        assert!(admit(&busy, &sender, 17));
        assert!(busy.load(Ordering::SeqCst));
        assert!(!admit(&busy, &sender, 19));
        assert_eq!(receiver.try_recv().unwrap(), 17);
        busy.store(false, Ordering::SeqCst);
        drop(receiver);
        assert!(!admit(&busy, &sender, 23));
        assert!(!busy.load(Ordering::SeqCst));
        let (sender, _receiver) = mpsc::sync_channel(1);
        sender.send(29).unwrap();
        assert!(!admit(&busy, &sender, 31));
        assert!(!busy.load(Ordering::SeqCst));
    }
    #[test]
    fn accessibility_trust_alone_authorizes_capture_and_posting() {
        for mask in 0..8 {
            let state = PermissionState {
                accessibility: mask & 1 != 0,
                listen: mask & 2 != 0,
                post: mask & 4 != 0,
            };
            assert_eq!(state.ready(), mask & 1 != 0, "mask {mask}");
        }
        // Listening and posting follow from Accessibility; Input Monitoring is
        // only needed for processes macOS does not trust for Accessibility.
        assert!(
            PermissionState {
                accessibility: true,
                listen: false,
                post: false
            }
            .ready()
        );
    }
    #[test]
    fn cancellation_target_and_secure_input_refuse_effects() {
        assert!(context_valid(true, 7, 7, true, false));
        for args in [
            (false, 7, 7, true, false),
            (true, 8, 7, true, false),
            (true, 7, 7, false, false),
            (true, 7, 7, true, true),
        ] {
            assert!(!context_valid(args.0, args.1, args.2, args.3, args.4));
        }
    }
    #[test]
    fn concurrent_copy_or_missing_marker_invalidates_lease() {
        assert!(lease_matches(15, 15, true));
        assert!(!lease_matches(16, 15, true));
        assert!(!lease_matches(15, 15, false));
    }
    #[test]
    fn unicode_events_keep_surrogate_pairs_and_combining_scalars_whole() {
        let chunks = unicode_events("a👋e\u{301}");
        assert_eq!(
            chunks,
            vec![vec![97], vec![0xd83d, 0xdc4b], vec![101], vec![0x301]]
        );
        assert!(unicode_events("").is_empty());
    }
}
