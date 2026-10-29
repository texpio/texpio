//! Pure policy shared by macOS capture and effects; runnable without native input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PermissionState {
    pub accessibility: bool,
    pub listen: bool,
    pub post: bool,
}
impl PermissionState {
    pub fn ready(self) -> bool {
        self.accessibility && self.listen && self.post
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
pub(crate) fn context_evidence_safe(subrole: Option<&str>, marked: Option<&str>) -> bool {
    matches!(subrole, Some("AXSearchField")) && marked == Some("")
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
    fn unknown_protection_or_composition_evidence_refuses_context() {
        assert!(context_evidence_safe(Some("AXSearchField"), Some("")));
        for (subrole, marked) in [
            (None, Some("")),
            (Some("AXSearchField"), None),
            (None, None),
            (Some("AXSecureTextField"), Some("")),
            (Some("AXUnknown"), Some("")),
            (Some(""), Some("")),
            (Some("AXSearchField"), Some("composing")),
        ] {
            assert!(!context_evidence_safe(subrole, marked));
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
    fn every_permission_is_required_without_prompting() {
        for mask in 0..8 {
            assert_eq!(
                PermissionState {
                    accessibility: mask & 1 != 0,
                    listen: mask & 2 != 0,
                    post: mask & 4 != 0
                }
                .ready(),
                mask == 7
            );
        }
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
