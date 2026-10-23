use crate::{
    shortcuts::{DEFAULT_SHORTCUT, ShortcutActor, ShortcutBackend, ShortcutController},
    types::ServiceError,
};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, atomic::AtomicU64},
};

#[derive(Default)]
struct State {
    bound: HashSet<String>,
    saved: Option<String>,
    registrations: usize,
    conflict: bool,
    save_fail: usize,
    unregister_fail: usize,
    save_calls: usize,
    fail_save_at: Option<usize>,
}
#[derive(Clone)]
struct Fake(Arc<Mutex<State>>);
impl ShortcutBackend for Fake {
    fn canonicalize(&self, value: &str) -> Result<String, ServiceError> {
        if !value.starts_with("CommandOrControl+") || value.len() > 128 {
            return Err(ServiceError::InvalidInput);
        }
        Ok(value.to_string())
    }
    fn register(&mut self, chord: &str, _: u64) -> Result<(), ServiceError> {
        let mut state = self.0.lock().unwrap();
        if state.conflict || !state.bound.insert(chord.into()) {
            return Err(ServiceError::Unavailable);
        }
        state.registrations += 1;
        Ok(())
    }
    fn unregister(&mut self, chord: &str) -> Result<(), ServiceError> {
        let mut state = self.0.lock().unwrap();
        if state.unregister_fail > 0 {
            state.unregister_fail -= 1;
            return Err(ServiceError::Unavailable);
        }
        state.bound.remove(chord);
        Ok(())
    }
    fn load(&self) -> Result<Option<String>, ServiceError> {
        Ok(self.0.lock().unwrap().saved.clone())
    }
    fn save(&mut self, chord: Option<String>) -> Result<(), ServiceError> {
        let mut state = self.0.lock().unwrap();
        state.save_calls += 1;
        if state.fail_save_at == Some(state.save_calls) {
            return Err(ServiceError::Io);
        }
        if state.save_fail > 0 {
            state.save_fail -= 1;
            return Err(ServiceError::Io);
        }
        state.saved = chord;
        Ok(())
    }
}
fn fixture() -> (ShortcutController<Fake>, Arc<Mutex<State>>) {
    let state = Arc::new(Mutex::new(State::default()));
    let mut controller = ShortcutController::new(Fake(state.clone()), Arc::new(AtomicU64::new(0)));
    controller.initialize().unwrap();
    (controller, state)
}
const NEXT: &str = "CommandOrControl+Shift+J";
#[test]
fn conflict_keeps_previous_binding() {
    let (mut controller, state) = fixture();
    state.lock().unwrap().conflict = true;
    assert_eq!(controller.replace(NEXT), Err(ServiceError::Unavailable));
    let state = state.lock().unwrap();
    assert_eq!(state.saved, None);
    assert_eq!(state.bound, HashSet::from([DEFAULT_SHORTCUT.into()]));
    assert_eq!(state.registrations, 1);
}
#[test]
fn malformed_and_same_chords_do_not_install_handlers() {
    let (mut controller, state) = fixture();
    assert_eq!(
        controller.replace("plain key"),
        Err(ServiceError::InvalidInput)
    );
    controller.replace(DEFAULT_SHORTCUT).unwrap();
    assert_eq!(state.lock().unwrap().registrations, 1);
}
#[test]
fn save_failure_rolls_back_new_binding_and_can_retry() {
    let (mut controller, state) = fixture();
    state.lock().unwrap().save_fail = 1;
    assert_eq!(controller.replace(NEXT), Err(ServiceError::Io));
    assert_eq!(
        state.lock().unwrap().bound,
        HashSet::from([DEFAULT_SHORTCUT.into()])
    );
    controller.replace(NEXT).unwrap();
    let state = state.lock().unwrap();
    assert_eq!(state.saved.as_deref(), Some(NEXT));
    assert_eq!(state.bound, HashSet::from([NEXT.into()]));
}
#[test]
fn unregister_failure_preserves_active_chord_and_saved_preference() {
    let (mut controller, state) = fixture();
    state.lock().unwrap().unregister_fail = 1;
    assert_eq!(controller.replace(NEXT), Err(ServiceError::Unavailable));
    assert_eq!(state.lock().unwrap().saved, None);
    assert_eq!(
        state.lock().unwrap().bound,
        HashSet::from([DEFAULT_SHORTCUT.into()])
    );
    controller.replace(NEXT).unwrap();
}
#[test]
fn failed_rollback_cleanup_remains_recoverable() {
    let (mut controller, state) = fixture();
    {
        let mut state = state.lock().unwrap();
        state.save_fail = 1;
        state.unregister_fail = 1;
    }
    assert!(controller.replace(NEXT).is_err());
    controller.replace(NEXT).unwrap();
    assert_eq!(state.lock().unwrap().bound, HashSet::from([NEXT.into()]));
}
#[test]
fn actor_serializes_concurrent_changes() {
    let state = Arc::new(Mutex::new(State::default()));
    let actor = ShortcutActor::spawn(Fake(state.clone()), Arc::new(AtomicU64::new(0)));
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let actor = actor.clone();
            std::thread::spawn(move || actor.replace(format!("CommandOrControl+Shift+F{i}")))
        })
        .collect();
    for thread in threads {
        thread.join().unwrap().unwrap();
    }
    let state = state.lock().unwrap();
    assert_eq!(state.bound.len(), 1);
    assert!(state.bound.contains(state.saved.as_ref().unwrap()));
}
#[test]
fn rollback_persistence_failure_is_reported_and_repaired_on_retry() {
    let (mut controller, state) = fixture();
    {
        let mut state = state.lock().unwrap();
        state.unregister_fail = 1;
        state.fail_save_at = Some(2);
    }
    assert_eq!(controller.replace(NEXT), Err(ServiceError::Unavailable));
    controller.replace(DEFAULT_SHORTCUT).unwrap();
    let state = state.lock().unwrap();
    assert_eq!(state.saved, None);
    assert_eq!(state.bound, HashSet::from([DEFAULT_SHORTCUT.into()]));
}
#[test]
fn startup_conflict_does_not_kill_actor_and_can_be_retried() {
    let state = Arc::new(Mutex::new(State {
        conflict: true,
        ..State::default()
    }));
    let actor = ShortcutActor::spawn(Fake(state.clone()), Arc::new(AtomicU64::new(0)));
    assert_eq!(actor.replace(NEXT.into()), Err(ServiceError::Unavailable));
    state.lock().unwrap().conflict = false;
    actor.replace(NEXT.into()).unwrap();
    assert_eq!(state.lock().unwrap().bound, HashSet::from([NEXT.into()]));
}
