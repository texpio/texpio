use std::io::Cursor;
use texpio_wayland_helper::*;

fn session() -> Session {
    Session {
        uid: 1000,
        pid: 42,
        id: "3".into(),
        seat: "seat0".into(),
        active: true,
        local: true,
        user: true,
        wayland: true,
        unlocked: true,
    }
}
#[test]
fn authorizes_only_matching_active_local_unlocked_user_session() {
    let expected = session();
    assert!(authorize(&expected, &expected));
    for field in 0..9 {
        let mut actual = expected.clone();
        match field {
            0 => actual.uid += 1,
            1 => actual.pid += 1,
            2 => actual.id = "4".into(),
            3 => actual.seat = "seat1".into(),
            4 => actual.active = false,
            5 => actual.local = false,
            6 => actual.user = false,
            7 => actual.wayland = false,
            _ => actual.unlocked = false,
        }
        assert!(!authorize(&expected, &actual), "field {field}");
    }
}
#[test]
fn framing_rejects_unbounded_malformed_and_unknown_versions() {
    assert!(read_frame::<Request>(&mut Cursor::new((MAX_FRAME as u32 + 1).to_be_bytes())).is_err());
    assert!(read_frame::<Request>(&mut Cursor::new([0, 0, 0, 1, b'{'])).is_err());
    let mut bytes = Vec::new();
    write_frame(
        &mut bytes,
        &Request::Hello {
            version: VERSION + 1,
        },
    )
    .unwrap();
    let frame: Request = read_frame(&mut Cursor::new(bytes)).unwrap();
    assert!(!frame.valid_hello());
}
#[test]
fn generations_deadlines_disconnect_and_revocation_cannot_resume_jobs() {
    let mut lease = Lease::new(7, 100);
    assert!(lease.accept(7, 101, 200));
    assert!(!lease.accept(6, 101, 200));
    assert!(!lease.accept(7, 201, 200));
    assert!(!lease.accept(7, 101, 10_000));
    lease.revoke();
    assert!(!lease.accept(7, 102, 200));
    assert!(!lease.heartbeat(7, 102));
    let mut next = Lease::new(8, 102);
    assert!(!next.accept(7, 103, 200));
    assert!(!next.accept(8, 3103, 3200));
}
fn field() -> FieldEvidence {
    FieldEvidence {
        token: 3,
        context_epoch: 2,
        pid: 23,
        caret: 12,
        suffix: "prefix;hello".into(),
        focused: true,
        editable: true,
        protected: false,
        composition: Composition::Idle,
        selections: 0,
    }
}
#[test]
fn raw_final_key_is_not_commit_proof() {
    let before = field();
    assert!(before.verify(3, 2, 23, ";hello").is_ok());
    let mut delayed = before.clone();
    delayed.suffix.pop();
    delayed.caret -= 1;
    assert!(delayed.verify(3, 2, 23, ";hello").is_err());
    for choice in 0..8 {
        let mut bad = before.clone();
        match choice {
            0 => bad.focused = false,
            1 => bad.editable = false,
            2 => bad.protected = true,
            3 => bad.composition = Composition::Unknown,
            4 => bad.selections = 1,
            5 => bad.token += 1,
            6 => bad.context_epoch += 1,
            _ => bad.pid += 1,
        }
        assert!(bad.verify(3, 2, 23, ";hello").is_err());
    }
    assert!(before.verify(3, 2, 23, "é").is_err());
    assert!(before.verify(3, 2, 23, "hello\n").is_err());
}
#[test]
fn erased_caret_must_be_observed_before_payload() {
    let before = field();
    let mut after = before.clone();
    after.caret -= 6;
    after.suffix = "prefix".into();
    assert!(after.erasure_observed(&before, 6));
    assert!(!before.erasure_observed(&before, 6));
    after.token += 1;
    assert!(!after.erasure_observed(&before, 6));
}

#[test]
fn erasure_evidence_handles_unicode_prefix_without_splitting_utf8() {
    let mut before = field();
    before.suffix = "👋;hello".into();
    before.caret = 7;
    let mut after = before.clone();
    after.suffix = "👋".into();
    after.caret = 1;
    assert!(after.erasure_observed(&before, 6));
    let mut malformed = before.clone();
    malformed.suffix = "👋".into();
    malformed.caret = after.caret + 1;
    assert!(!after.erasure_observed(&malformed, 1));
}
#[test]
fn unknown_fields_paths_and_commands_are_not_protocol_operations() {
    for value in [
        r#"{"op":"hello","version":1,"path":"/dev/input/event0"}"#,
        r#"{"op":"exec","command":"whoami"}"#,
        r#"{"op":"poll","generation":-1}"#,
    ] {
        let mut frame = (value.len() as u32).to_be_bytes().to_vec();
        frame.extend_from_slice(value.as_bytes());
        assert!(read_frame::<Request>(&mut Cursor::new(frame)).is_err());
    }
}
#[test]
fn overflow_and_virtual_devices_require_fresh_state() {
    assert!(approved_device(
        "/sys/devices/pci0000:00/input/input1/event1",
        "E:ID_INPUT_KEYBOARD=1\n"
    ));
    assert!(!approved_device(
        "/sys/devices/virtual/input/input1/event1",
        "E:ID_INPUT_KEYBOARD=1\n"
    ));
    assert!(!approved_device(
        "/sys/devices/pci0000:00/input/input1/event1",
        "E:ID_INPUT_KEYBOARD=1\nE:ID_SEAT=seat1\n"
    ));
    assert!(!approved_device("/tmp/event1", "E:ID_INPUT_KEYBOARD=1\n"));
    assert!(!approved_device(
        "/sys/devices/pci0000:00/input/input1/event1",
        ""
    ));
}

#[test]
fn uncertain_synthetic_transport_releases_and_rejects_every_later_job() {
    use std::{cell::RefCell, rc::Rc};
    type Calls = Rc<RefCell<Vec<Option<(u16, bool)>>>>;
    struct Sink {
        calls: Calls,
        fail: usize,
    }
    impl KeyOutput for Sink {
        fn event(&mut self, code: u16, down: bool) -> std::io::Result<()> {
            self.calls.borrow_mut().push(Some((code, down)));
            if self.calls.borrow().len() == self.fail {
                Err(std::io::ErrorKind::BrokenPipe.into())
            } else {
                Ok(())
            }
        }
        fn sync(&mut self) -> std::io::Result<()> {
            self.calls.borrow_mut().push(None);
            if self.calls.borrow().len() == self.fail {
                Err(std::io::ErrorKind::BrokenPipe.into())
            } else {
                Ok(())
            }
        }
    }
    for fail in 1..=4 {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut keyboard = SyntheticKeys::new(Sink {
            calls: calls.clone(),
            fail,
        });
        assert!(keyboard.tap(14).is_err());
        assert!(keyboard.poisoned());
        assert!(
            calls.borrow()[fail..].contains(&Some((14, false))),
            "failure stage {fail} must attempt release"
        );
        let count = calls.borrow().len();
        assert!(keyboard.tap(30).is_err());
        assert_eq!(
            calls.borrow().len(),
            count,
            "uncertainty must reject the next job"
        );
    }
}

#[test]
fn registry_root_pid_does_not_hide_the_foreground_application() {
    assert!(traverse_children(0, 77, 23));
    assert!(traverse_children(1, 23, 23));
    assert!(!traverse_children(1, 77, 23));
}
#[test]
fn physical_action_gate_only_consumes_the_expected_trigger_release_once() {
    let mut trigger = ObservationGate::trigger_release(30);
    assert!(trigger.accept(0, 0, 0));
    assert!(trigger.accept(1, 30, 0));
    assert!(!trigger.accept(1, 30, 0));
    for (kind, code, value) in [
        (1, 1, 1),
        (1, 1, 0),
        (1, 30, 1),
        (1, 30, 2),
        (2, 0, 1),
        (1, 272, 0),
        (0, 3, 0),
    ] {
        assert!(!ObservationGate::quiet().accept(kind, code, value));
        assert!(!ObservationGate::trigger_release(30).accept(kind, code, value));
    }
}
