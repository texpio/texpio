use super::{context::PreparedEdit, current_stamp};
use crate::engine::{runtime::DispatchCounts, types::RefusalReason};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_BACK,
};

pub const OWN_INPUT_MARKER: usize = 0x54585050;

pub struct InjectionPlan {
    events: Vec<INPUT>,
}

pub fn unicode_plan(edit: &PreparedEdit) -> Result<InjectionPlan, RefusalReason> {
    build_plan(edit.delete_count(), edit.output())
}

fn build_plan(delete_count: usize, output: &str) -> Result<InjectionPlan, RefusalReason> {
    let units = output.encode_utf16().count();
    if delete_count > 33
        || units == 0
        || units > 32_769
        || output
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\t' | '\n' | '\r'))
    {
        return Err(RefusalReason::ContextUnavailable);
    }
    let mut events = Vec::with_capacity((delete_count + units) * 2);
    for _ in 0..delete_count {
        events.push(key(VK_BACK, 0, KEYBD_EVENT_FLAGS(0)));
        events.push(key(VK_BACK, 0, KEYEVENTF_KEYUP));
    }
    // Literal UTF-16, including LF. Accepted CRLF conversions require a separately
    // qualified profile contract; neither encoding nor verification guesses one.
    for unit in output.encode_utf16() {
        events.push(key(VIRTUAL_KEY(0), unit, KEYEVENTF_UNICODE));
        events.push(key(
            VIRTUAL_KEY(0),
            unit,
            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        ));
    }
    Ok(InjectionPlan { events })
}

fn key(virtual_key: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: virtual_key,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: OWN_INPUT_MARKER,
            },
        },
    }
}

/// The worker calls this only inside the dispatch gate, after UIA revalidation.
/// One batch is attempted; a partial result is never repaired or replayed.
pub fn dispatch(
    edit: &PreparedEdit,
    plan: &InjectionPlan,
) -> Result<DispatchCounts, RefusalReason> {
    if current_stamp()? != edit.stamp() {
        return Err(RefusalReason::StaleContext);
    }
    // SAFETY: initialized bounded INPUT slice; no borrowed pointers survive call.
    // This API has no HWND and is not atomic with the preceding focus check.
    let submitted = unsafe { SendInput(&plan.events, std::mem::size_of::<INPUT>() as i32) };
    Ok(DispatchCounts {
        expected: plan.events.len() as u32,
        submitted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_and_unicode_are_one_tagged_bounded_batch() {
        let plan = build_plan(7, "Hello from Texpio! ").unwrap();
        assert_eq!(
            plan.events.len(),
            14 + "Hello from Texpio! ".encode_utf16().count() * 2
        );
        for (index, event) in plan.events.iter().enumerate() {
            assert_eq!(event.r#type, INPUT_KEYBOARD);
            let key = unsafe { event.Anonymous.ki };
            assert_eq!(key.dwExtraInfo, OWN_INPUT_MARKER);
            if index < 14 {
                assert_eq!(key.wVk, VK_BACK);
            }
            assert_eq!(key.dwFlags.contains(KEYEVENTF_KEYUP), index % 2 == 1);
        }
    }

    #[test]
    fn surrogate_pairs_combining_marks_and_devanagari_remain_exact() {
        let body = "नमस्ते 👋 e\u{301} Café ";
        let plan = build_plan(0, body).unwrap();
        let units: Vec<u16> = plan
            .events
            .iter()
            .step_by(2)
            .map(|event| unsafe { event.Anonymous.ki.wScan })
            .collect();
        assert_eq!(units, body.encode_utf16().collect::<Vec<_>>());
    }

    #[test]
    fn line_feed_is_literal_and_overlarge_plans_are_refused() {
        let plan = build_plan(0, "one\ntwo ").unwrap();
        let units: Vec<u16> = plan
            .events
            .iter()
            .step_by(2)
            .map(|event| unsafe { event.Anonymous.ki.wScan })
            .collect();
        assert!(units.contains(&10));
        assert!(!units.contains(&13));
        assert!(build_plan(34, "body ").is_err());
        assert!(build_plan(0, &"x".repeat(32_770)).is_err());
        assert!(build_plan(0, "").is_err());
    }

    #[test]
    fn maximum_saved_unicode_body_is_bounded_and_control_packets_are_refused() {
        assert!(build_plan(33, &format!("{} ", "👋".repeat(16_384))).is_ok());
        for body in ["bad\u{8}body", "bad\u{1b}body", "bad\u{7f}body", "\0"] {
            assert!(build_plan(0, body).is_err());
        }
    }
}
