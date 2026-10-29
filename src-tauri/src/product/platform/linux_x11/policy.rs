//! Side-effect-free limits and lease policy; tested without connecting to X.
pub(crate) const MAX_FORMATS: usize = 32;
pub(crate) const MAX_BYTES: usize = 8 * 1024 * 1024;

pub(crate) fn native_x11(session: Option<&str>, wayland: bool) -> bool {
    session == Some("x11") && !wayland
}

pub(crate) fn snapshot_complete(formats: usize, bytes: usize, complete: bool) -> bool {
    complete && formats <= MAX_FORMATS && bytes <= MAX_BYTES
}

pub(crate) fn lease_current(ours: u32, owner: u32, acquired: u32, current: u32) -> bool {
    ours != 0 && ours == owner && acquired == current
}

pub(crate) fn ascii_keysym(symbol: u64) -> Option<char> {
    match symbol {
        0x20..=0x7e => char::from_u32(symbol as u32),
        0xff09 => Some('\t'),
        0xff0d => Some('\n'),
        _ => None,
    }
}

pub(crate) fn fake_inputs(
    mut data: &[u8],
    swapped: bool,
    opcode: u8,
    ours: u32,
    sender: u32,
) -> Result<Vec<(u8, u8)>, ()> {
    if data.len() > 64 * 1024 || ours != sender {
        return Err(());
    }
    let mut result = Vec::new();
    while data.len() >= 6 {
        let mut length = u16::from_ne_bytes([data[2], data[3]]);
        if swapped {
            length = length.swap_bytes();
        }
        let bytes = usize::from(length) * 4;
        if bytes < 6 || bytes > data.len() {
            return Err(());
        }
        if data[0] == opcode && data[1] == 2 {
            if result.len() >= 64 {
                return Err(());
            }
            result.push((data[4], data[5]));
        }
        data = &data[bytes..];
    }
    if data.is_empty() { Ok(result) } else { Err(()) }
}

pub(crate) fn prepare_text<T>(text: &str, lookup: impl FnMut(char) -> Option<T>) -> Option<Vec<T>> {
    if text.len() > MAX_BYTES {
        return None;
    }
    text.chars().map(lookup).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SelectionStamp {
    pub owner: u32,
    pub epoch: u32,
}
pub(crate) fn acquisition_current(expected: SelectionStamp, current: SelectionStamp) -> bool {
    expected == current
}

pub(crate) fn key_for(
    character: char,
    first: u8,
    last: u8,
    mut lookup: impl FnMut(u8, bool) -> Option<char>,
) -> Option<(u8, bool)> {
    for shift in [false, true] {
        for code in first..=last {
            if lookup(code, shift) == Some(character) {
                return Some((code, shift));
            }
        }
    }
    None
}

#[derive(Default)]
pub(crate) struct SyntheticKeys {
    held: std::collections::BTreeSet<u8>,
    uncertain: bool,
}
impl SyntheticKeys {
    pub fn before_send(&mut self, key: u8, down: bool) {
        if down {
            self.held.insert(key);
        }
    }
    pub fn acknowledge(&mut self, key: u8, down: bool, success: bool) {
        if !success {
            self.uncertain = true;
        } else if !down {
            self.held.remove(&key);
        }
    }
    pub fn held(&self) -> Vec<u8> {
        self.held.iter().copied().collect()
    }
    pub fn uncertain(&self) -> bool {
        self.uncertain
    }
    pub fn mark_uncertain(&mut self) {
        self.uncertain = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_xwayland_and_unknown_session() {
        assert!(!native_x11(Some("wayland"), true));
        assert!(!native_x11(None, false));
        assert!(native_x11(Some("x11"), false));
    }
    #[test]
    fn incomplete_or_oversize_clipboard_cannot_authorize_deletion() {
        assert!(!snapshot_complete(1, 10, false));
        assert!(!snapshot_complete(MAX_FORMATS + 1, 10, true));
        assert!(!snapshot_complete(1, MAX_BYTES + 1, true));
        assert!(snapshot_complete(2, 1024, true));
    }
    #[test]
    fn restored_selection_requires_same_owner_and_epoch() {
        assert!(!lease_current(9, 10, 20, 20));
        assert!(!lease_current(9, 9, 20, 21));
        assert!(!lease_current(0, 0, 20, 20));
        assert!(lease_current(9, 9, 20, 20));
    }
    #[test]
    fn composition_and_unicode_triggers_are_outside_initial_deletion_profile() {
        assert_eq!(ascii_keysym(0xfe51), None); // dead acute
        assert_eq!(ascii_keysym(0xff20), None); // Multi_key
        assert_eq!(ascii_keysym(0x010000e9), None);
        assert_eq!(ascii_keysym(0x3b), Some(';'));
    }
    #[test]
    fn record_batch_attributes_all_generated_keys_to_exact_connection() {
        let bytes = [150, 2, 2, 0, 2, 38, 0, 0, 150, 2, 2, 0, 3, 38, 0, 0];
        assert_eq!(
            fake_inputs(&bytes, false, 150, 123, 123),
            Ok(vec![(2, 38), (3, 38)])
        );
        assert_eq!(fake_inputs(&bytes, false, 150, 123, 124), Err(()));
        assert_eq!(fake_inputs(&bytes[..7], false, 150, 123, 123), Err(()));
    }
    #[test]
    fn swapped_record_client_and_truncated_requests_are_bounded() {
        let swapped = [150, 2, 0, 2, 2, 38, 0, 0];
        assert_eq!(
            fake_inputs(&swapped, true, 150, 123, 123),
            Ok(vec![(2, 38)])
        );
        assert_eq!(
            fake_inputs(&[150, 2, 0, 0, 2, 38], false, 150, 123, 123),
            Err(())
        );
    }
    #[test]
    fn entire_type_payload_must_be_representable_before_plan_is_returned() {
        let lookup = |ch| match ch {
            'A' => Some((38, true)),
            'é' => Some((26, false)),
            _ => None,
        };
        assert_eq!(
            prepare_text("Aé", lookup),
            Some(vec![(38, true), (26, false)])
        );
        assert_eq!(prepare_text("Aé🙂", lookup), None);
        assert_eq!(prepare_text("A\n", lookup), None);
    }
    #[test]
    fn acquisition_refuses_copy_after_snapshot_even_empty_or_self_owned() {
        for owner in [0, 9, 99] {
            let captured = SelectionStamp { owner, epoch: 20 };
            assert!(acquisition_current(captured, captured));
            assert!(!acquisition_current(
                captured,
                SelectionStamp {
                    owner: 100,
                    epoch: 21
                }
            ));
            assert!(!acquisition_current(
                captured,
                SelectionStamp { owner, epoch: 21 }
            ));
        }
    }
    #[test]
    fn unmapped_keycodes_do_not_hide_a_later_representable_key() {
        assert_eq!(
            key_for('v', 8, 11, |key, shift| match (key, shift) {
                (10, false) => Some('v'),
                _ => None,
            }),
            Some((10, false))
        );
    }
    #[test]
    fn failed_keyup_retains_key_for_cleanup_and_sticks_session_fault() {
        let mut keys = SyntheticKeys::default();
        keys.before_send(38, true);
        keys.acknowledge(38, true, true);
        keys.before_send(38, false);
        keys.acknowledge(38, false, false);
        assert_eq!(keys.held(), vec![38]);
        assert!(keys.uncertain());
        keys.acknowledge(38, false, true);
        assert!(keys.held().is_empty());
        assert!(
            keys.uncertain(),
            "a later successful release cannot authorize another job"
        );
    }
    #[test]
    fn failed_keydown_is_possibly_held_until_a_release_is_acknowledged() {
        let mut keys = SyntheticKeys::default();
        keys.before_send(50, true);
        keys.acknowledge(50, true, false);
        assert_eq!(keys.held(), vec![50]);
        assert!(keys.uncertain());
        keys.acknowledge(50, false, true);
        assert!(keys.held().is_empty());
    }
}
