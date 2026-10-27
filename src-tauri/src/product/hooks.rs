//! Explicitly started Windows hook and bounded effect worker.
use super::{
    effects::ProductError,
    matcher::{Expansion, ProductRule, TypedHistory},
    service::Shared,
    settings::ProductSettings,
    windows::{NativeEdit, PRODUCT_MARKER},
};
use crate::engine::windows::{TargetStamp, current_stamp};
use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::{
        LibraryLoader::GetModuleHandleW,
        Threading::{GetCurrentProcessId, GetCurrentThreadId},
    },
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

static HOOK_OWNED: AtomicBool = AtomicBool::new(false);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputOrigin {
    Own,
    Hardware,
    Software,
    LowerIntegrity,
}
fn input_origin(event: &KBDLLHOOKSTRUCT) -> InputOrigin {
    if event.dwExtraInfo == PRODUCT_MARKER {
        InputOrigin::Own
    } else if event.flags.contains(LLKHF_LOWER_IL_INJECTED) {
        InputOrigin::LowerIntegrity
    } else if event.flags.contains(LLKHF_INJECTED) {
        InputOrigin::Software
    } else {
        InputOrigin::Hardware
    }
}
fn accepts_origin(origin: InputOrigin, allow_software: bool) -> bool {
    origin == InputOrigin::Hardware || (origin == InputOrigin::Software && allow_software)
}
// Explicit debug-only capture diagnostics contain counters, never keys or text.
#[cfg(debug_assertions)]
#[derive(Debug, Default)]
struct CaptureFacts {
    callbacks: u32,
    own_events: u32,
    injected_events: u32,
    lower_integrity_events: u32,
    software_accepted: u32,
    key_down: u32,
    generic_modifiers: u32,
    sided_modifiers: u32,
    stamp_refused: u32,
    context_resets: u32,
    own_target: u32,
    shortcut_refused: u32,
    translation_failed: u32,
    translated_scalars: u32,
    spaces: u32,
    matches: u32,
    accepted: u32,
    callback_max_us: u64,
}
macro_rules! fact {
    ($context:expr, $name:ident) => {
        #[cfg(debug_assertions)]
        if let Some(facts) = $context.facts.as_mut() {
            facts.$name = facts.$name.saturating_add(1);
        }
    };
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn software_compatibility_is_explicit_and_never_accepts_own_or_lower_integrity_input() {
        assert!(accepts_origin(InputOrigin::Hardware, false));
        assert!(accepts_origin(InputOrigin::Hardware, true));
        assert!(!accepts_origin(InputOrigin::Software, false));
        assert!(accepts_origin(InputOrigin::Software, true));
        for allowed in [false, true] {
            assert!(!accepts_origin(InputOrigin::Own, allowed));
            assert!(!accepts_origin(InputOrigin::LowerIntegrity, allowed));
        }
        let event = KBDLLHOOKSTRUCT {
            flags: LLKHF_INJECTED,
            ..Default::default()
        };
        assert_eq!(input_origin(&event), InputOrigin::Software);
        assert_eq!(
            input_origin(&KBDLLHOOKSTRUCT {
                flags: LLKHF_INJECTED | LLKHF_LOWER_IL_INJECTED,
                ..event
            }),
            InputOrigin::LowerIntegrity
        );
        assert_eq!(
            input_origin(&KBDLLHOOKSTRUCT {
                dwExtraInfo: PRODUCT_MARKER,
                ..event
            }),
            InputOrigin::Own
        );
    }
    #[test]
    fn translation_accepts_one_unicode_scalar_but_never_a_partial_composition() {
        assert_eq!(translated_character("é"), Some('é'));
        assert_eq!(translated_character("👋"), Some('👋'));
        assert_eq!(translated_character("\r"), Some('\n'));
        assert_eq!(translated_character(""), None);
        assert_eq!(translated_character("ab"), None);
        assert_eq!(translated_character("e\u{301}"), None);
    }
    #[test]
    fn completing_key_is_suppressed_only_when_its_exact_job_is_accepted() {
        for (busy, disconnected) in [(false, false), (true, false), (false, true)] {
            let engine = super::super::service::ProductEngine::new();
            engine.shared.busy.store(busy, Ordering::SeqCst);
            let (sender, receiver) = mpsc::sync_channel(1);
            let mut receiver = Some(receiver);
            if disconnected {
                receiver.take();
            }
            let mut context = HookContext {
                shared: engine.shared.clone(),
                jobs: sender,
                history: TypedHistory::new(
                    vec![],
                    super::super::matcher::Boundary::Word,
                    super::super::matcher::Activation::Immediate,
                ),
                settings: ProductSettings::default(),
                keys: [0; 256],
                last_target: None,
                generation: 7,
                swallowed_cancel: None,
                #[cfg(debug_assertions)]
                facts: None,
            };
            let target = TargetStamp {
                foreground_window: 1,
                focused_window: 2,
                process_id: 3,
                thread_id: 4,
                keyboard_layout: 5,
            };
            let expansion = Expansion {
                rule: ProductRule {
                    id: "test".into(),
                    trigger: ";hello".into(),
                    body: "Hello!".into(),
                    rich_html: None,
                },
                erase_count: 5,
                terminator: None,
            };
            let accepted = submit_match(&mut context, expansion.clone(), target, 7);
            assert_eq!(accepted, !busy && !disconnected);
            assert_eq!(engine.shared.busy.load(Ordering::SeqCst), busy || accepted);
            if accepted {
                let job = receiver.unwrap().try_recv().unwrap();
                assert_eq!(job.expansion, expansion);
                assert!(job.target == target);
                assert_eq!(job.ticket, 7);
            } else if let Some(receiver) = receiver {
                assert!(receiver.try_recv().is_err());
            }
        }
    }
    #[test]
    fn initial_modifier_seed_reconciles_generic_bits_with_observed_sides() {
        let mut keys = [0; 256];
        for generic in [0x10, 0x11, 0x12] {
            keys[generic] = 0x80;
        }
        keys[0x14] = 1;
        seed_modifiers(&mut keys, |_| false);
        for generic in [0x10, 0x11, 0x12] {
            assert_eq!(keys[generic], 0);
        }
        seed_modifiers(&mut keys, |key| matches!(key, 0xA0 | 0xA3));
        assert_eq!(keys[0x10], 0x80);
        assert_eq!(keys[0x11], 0x80);
        assert_eq!(keys[0x12], 0);
        assert_eq!(keys[0x14], 1);
    }
    #[test]
    fn lock_keys_keep_initial_toggle_and_do_not_toggle_on_repeat() {
        let mut keys = [0; 256];
        keys[VK_CAPITAL.0 as usize] = 1;
        update_keyboard(&mut keys, VK_CAPITAL.0 as usize, true);
        assert_eq!(keys[VK_CAPITAL.0 as usize] & 1, 0);
        update_keyboard(&mut keys, VK_CAPITAL.0 as usize, true);
        assert_eq!(keys[VK_CAPITAL.0 as usize] & 1, 0);
        update_keyboard(&mut keys, VK_CAPITAL.0 as usize, false);
        update_keyboard(&mut keys, VK_CAPITAL.0 as usize, true);
        assert_eq!(keys[VK_CAPITAL.0 as usize] & 1, 1);
    }
    #[test]
    fn generic_shift_stays_held_through_printable_keys_until_release() {
        let mut keys = [0; 256];
        update_keyboard(&mut keys, VK_SHIFT.0 as usize, true);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0x80);
        update_keyboard(&mut keys, 0xBA, true);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0x80);
        update_keyboard(&mut keys, 0xBA, false);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0x80);
        update_keyboard(&mut keys, VK_SHIFT.0 as usize, false);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0);
    }
    #[test]
    fn sided_shift_stays_held_until_both_sides_are_released() {
        let mut keys = [0; 256];
        update_keyboard(&mut keys, VK_LSHIFT.0 as usize, true);
        update_keyboard(&mut keys, VK_RSHIFT.0 as usize, true);
        update_keyboard(&mut keys, VK_LSHIFT.0 as usize, false);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0x80);
        update_keyboard(&mut keys, 0xBA, true);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0x80);
        update_keyboard(&mut keys, VK_RSHIFT.0 as usize, false);
        assert_eq!(keys[VK_SHIFT.0 as usize] & 0x80, 0);
    }
    #[test]
    fn injected_modifier_release_clears_cached_state_without_recognition() {
        let engine = super::super::service::ProductEngine::new();
        engine.shared.active.store(true, Ordering::SeqCst);
        let (sender, receiver) = mpsc::sync_channel(1);
        let mut history = TypedHistory::new(
            vec![ProductRule {
                id: "owned-state-regression".into(),
                trigger: ":probe".into(),
                body: "unused".into(),
                rich_html: None,
            }],
            super::super::matcher::Boundary::Word,
            super::super::matcher::Activation::Terminator,
        );
        for ch in ":probe".chars() {
            assert!(history.push(ch).is_none());
        }
        let mut keys = [0; 256];
        keys[VK_LCONTROL.0 as usize] = 0x80;
        keys[VK_CONTROL.0 as usize] = 0x80;
        let mut context = HookContext {
            shared: engine.shared.clone(),
            jobs: sender,
            history,
            settings: ProductSettings::default(),
            keys,
            last_target: None,
            generation: 0,
            swallowed_cancel: None,
            #[cfg(debug_assertions)]
            facts: None,
        };
        let event = KBDLLHOOKSTRUCT {
            vkCode: u32::from(VK_LCONTROL.0),
            flags: LLKHF_INJECTED | LLKHF_UP,
            ..Default::default()
        };
        assert!(!on_key(&mut context, event, WM_KEYUP));
        assert_eq!(engine.shared.generation.load(Ordering::SeqCst), 1);
        assert!(context.history.push(' ').is_none());
        assert!(receiver.try_recv().is_err());
        assert_eq!(context.keys[VK_LCONTROL.0 as usize] & 0x80, 0);
        assert_eq!(context.keys[VK_CONTROL.0 as usize] & 0x80, 0);
    }
    #[test]
    fn swallowed_cancel_keyup_is_consumed_even_after_effect_finishes() {
        let engine = super::super::service::ProductEngine::new();
        engine.shared.active.store(true, Ordering::SeqCst);
        engine.shared.busy.store(true, Ordering::SeqCst);
        let (sender, _receiver) = mpsc::sync_channel(1);
        let mut context = HookContext {
            shared: engine.shared.clone(),
            jobs: sender,
            history: TypedHistory::new(
                vec![],
                super::super::matcher::Boundary::Word,
                super::super::matcher::Activation::Immediate,
            ),
            settings: ProductSettings::default(),
            keys: [0; 256],
            last_target: None,
            generation: 0,
            swallowed_cancel: None,
            #[cfg(debug_assertions)]
            facts: None,
        };
        let event = KBDLLHOOKSTRUCT {
            vkCode: u32::from(VK_ESCAPE.0),
            ..Default::default()
        };
        assert!(on_key(&mut context, event, WM_KEYDOWN));
        engine.shared.busy.store(false, Ordering::SeqCst);
        assert!(on_key(&mut context, event, WM_KEYUP));
    }
    #[test]
    fn shortcut_backspace_discards_character_history_before_any_target_observation() {
        for modifier in [VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN] {
            let engine = super::super::service::ProductEngine::new();
            engine.shared.active.store(true, Ordering::SeqCst);
            let (sender, _receiver) = mpsc::sync_channel(1);
            let mut history = TypedHistory::new(
                vec![ProductRule {
                    id: "test".into(),
                    trigger: ";hello".into(),
                    body: "hello".into(),
                    rich_html: None,
                }],
                super::super::matcher::Boundary::Word,
                super::super::matcher::Activation::Immediate,
            );
            for ch in ";hellx".chars() {
                assert!(history.push(ch).is_none());
            }
            let mut context = HookContext {
                shared: engine.shared.clone(),
                jobs: sender,
                history,
                settings: ProductSettings::default(),
                keys: [0; 256],
                last_target: None,
                generation: 0,
                swallowed_cancel: None,
                #[cfg(debug_assertions)]
                facts: Some(CaptureFacts::default()),
            };
            context.keys[modifier.0 as usize] = 0x80;
            assert!(!on_key(
                &mut context,
                KBDLLHOOKSTRUCT {
                    vkCode: u32::from(VK_BACK.0),
                    ..Default::default()
                },
                WM_KEYDOWN
            ));
            assert!(context.history.push('o').is_none());
            #[cfg(debug_assertions)]
            {
                assert_eq!(context.facts.as_ref().unwrap().shortcut_refused, 1);
                assert_eq!(context.facts.as_ref().unwrap().stamp_refused, 0);
            }
        }
    }
}
struct Job {
    expansion: Expansion,
    target: TargetStamp,
    ticket: u64,
}
struct HookContext {
    shared: Arc<Shared>,
    jobs: SyncSender<Job>,
    history: TypedHistory,
    settings: ProductSettings,
    keys: [u8; 256],
    last_target: Option<TargetStamp>,
    generation: u64,
    swallowed_cancel: Option<u16>,
    #[cfg(debug_assertions)]
    facts: Option<CaptureFacts>,
}
thread_local! { static CONTEXT: RefCell<Option<HookContext>> = const { RefCell::new(None) }; }

pub(crate) struct HookRuntime {
    shared: Arc<Shared>,
    hook_id: u32,
    hook_done: Receiver<()>,
    worker_done: Receiver<()>,
    hook: Option<JoinHandle<()>>,
    worker: Option<JoinHandle<()>>,
}
impl HookRuntime {
    pub fn launch(
        shared: Arc<Shared>,
        rules: Vec<ProductRule>,
        settings: ProductSettings,
    ) -> Result<Self, ProductError> {
        if HOOK_OWNED.swap(true, Ordering::SeqCst) {
            return Err(ProductError::Busy);
        }
        let (jobs, received) = mpsc::sync_channel::<Job>(1);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (hook_done_tx, hook_done) = mpsc::sync_channel(1);
        let (worker_done_tx, worker_done) = mpsc::sync_channel(1);
        let worker_shared = shared.clone();
        let worker_settings = settings.clone();
        let worker = thread::Builder::new()
            .name("texpio-product-effects".into())
            .spawn(move || {
                while worker_shared.active.load(Ordering::SeqCst) {
                    let job = match received.recv_timeout(Duration::from_millis(25)) {
                        Ok(job) => job,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(_) => break,
                    };
                    let _effects = worker_shared
                        .effects
                        .lock()
                        .unwrap_or_else(|p| p.into_inner());
                    let resolver = worker_shared
                        .files
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .clone();
                    let files = resolver.map_or(Ok(Default::default()), |resolver| {
                        resolver(&job.expansion.rule.id)
                    });
                    let mut edit = files.and_then(|files| {
                        NativeEdit::new(
                            job.target,
                            worker_shared.generation.clone(),
                            job.ticket,
                            worker_settings.clone(),
                        )
                        .map(|edit| edit.with_files(files))
                    });
                    let outcome = match edit.as_mut() {
                        Ok(edit) => edit.deliver(&job.expansion),
                        Err(error) => Err(*error),
                    };
                    if outcome.is_ok() {
                        let recorder = worker_shared
                            .used
                            .lock()
                            .unwrap_or_else(|p| p.into_inner())
                            .clone();
                        if let Some(recorder) = recorder {
                            recorder(&job.expansion.rule.id);
                        }
                    }
                    {
                        let mut status = worker_shared
                            .status
                            .lock()
                            .unwrap_or_else(|p| p.into_inner());
                        if let Ok(edit) = edit {
                            status.submitted = status
                                .submitted
                                .saturating_add(u32::from(edit.submitted_events > 0));
                            status.clipboard_incomplete = edit.clipboard_incomplete;
                        }
                        status.last_error = outcome.err();
                        if status.last_error == Some(ProductError::Cancelled) {
                            status.cancelled = status.cancelled.saturating_add(1);
                        }
                    }
                    worker_shared.busy.store(false, Ordering::SeqCst);
                    worker_shared.generation.fetch_add(1, Ordering::SeqCst);
                }
                worker_shared.busy.store(false, Ordering::SeqCst);
                let _ = worker_done_tx.send(());
            })
            .map_err(|_| {
                HOOK_OWNED.store(false, Ordering::SeqCst);
                ProductError::Unavailable
            })?;
        let hook_shared = shared.clone();
        let hook = match thread::Builder::new()
            .name("texpio-product-hook".into())
            .spawn(move || {
                let generation = hook_shared.generation.load(Ordering::SeqCst);
                let history = TypedHistory::new(rules, settings.boundary, settings.activation);
                CONTEXT.with(|slot| {
                    *slot.borrow_mut() = Some(HookContext {
                        shared: hook_shared.clone(),
                        jobs,
                        history,
                        settings,
                        keys: initial_keyboard(),
                        last_target: None,
                        generation,
                        swallowed_cancel: None,
                        #[cfg(debug_assertions)]
                        facts: (std::env::var_os("TEXPIO_OWNED_CAPTURE_DIAGNOSTICS").as_deref()
                            == Some(std::ffi::OsStr::new("1")))
                        .then(CaptureFacts::default),
                    })
                });
                // SAFETY: callbacks/TLS stay alive on this thread until both hooks are removed.
                unsafe {
                    let mut message = MSG::default();
                    let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
                    let module = GetModuleHandleW(None).ok().map(Into::into);
                    let keyboard =
                        SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_callback), module, 0);
                    let pointer = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_callback), module, 0);
                    match (keyboard, pointer) {
                        (Ok(keyboard), Ok(pointer)) => {
                            let _ = ready_tx.send(Ok(GetCurrentThreadId()));
                            loop {
                                if !hook_shared.active.load(Ordering::SeqCst)
                                    || GetMessageW(&mut message, None, 0, 0).0 <= 0
                                {
                                    break;
                                }
                            }
                            let _ = UnhookWindowsHookEx(keyboard);
                            let _ = UnhookWindowsHookEx(pointer);
                        }
                        (keyboard, pointer) => {
                            if let Ok(hook) = keyboard {
                                let _ = UnhookWindowsHookEx(hook);
                            }
                            if let Ok(hook) = pointer {
                                let _ = UnhookWindowsHookEx(hook);
                            }
                            let _ = ready_tx.send(Err(ProductError::Unavailable));
                        }
                    }
                }
                CONTEXT.with(|slot| {
                    #[cfg(debug_assertions)]
                    if let Some(facts) = slot
                        .borrow()
                        .as_ref()
                        .and_then(|context| context.facts.as_ref())
                    {
                        eprintln!("Texpio owned capture facts: {facts:?}");
                    }
                    *slot.borrow_mut() = None;
                });
                HOOK_OWNED.store(false, Ordering::SeqCst);
                let _ = hook_done_tx.send(());
            }) {
            Ok(hook) => hook,
            Err(_) => {
                shared.active.store(false, Ordering::SeqCst);
                HOOK_OWNED.store(false, Ordering::SeqCst);
                return Err(ProductError::Unavailable);
            }
        };
        match ready_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(hook_id)) => Ok(Self {
                shared,
                hook_id,
                hook_done,
                worker_done,
                hook: Some(hook),
                worker: Some(worker),
            }),
            _ => {
                shared.active.store(false, Ordering::SeqCst);
                shared.generation.fetch_add(1, Ordering::SeqCst);
                Err(ProductError::Unavailable)
            }
        }
    }
    pub fn shutdown(&mut self) -> bool {
        self.shared.active.store(false, Ordering::SeqCst);
        self.shared.generation.fetch_add(1, Ordering::SeqCst);
        unsafe {
            let _ = PostThreadMessageW(self.hook_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if self.hook.is_some()
            && self
                .hook_done
                .recv_timeout(Duration::from_millis(500))
                .is_ok()
            && let Some(hook) = self.hook.take()
        {
            let _ = hook.join();
        }
        if self.worker.is_some()
            && self
                .worker_done
                .recv_timeout(Duration::from_millis(500))
                .is_ok()
            && let Some(worker) = self.worker.take()
        {
            let _ = worker.join();
        }
        self.hook.is_none() && self.worker.is_none()
    }
}
impl Drop for HookRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

unsafe extern "system" fn keyboard_callback(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    #[cfg(debug_assertions)]
    let callback_started = std::time::Instant::now();
    if code == HC_ACTION as i32 && data.0 != 0 {
        // SAFETY: hook owns the borrowed event for this callback only.
        let event = unsafe { *(data.0 as *const KBDLLHOOKSTRUCT) };
        #[cfg(debug_assertions)]
        CONTEXT.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut()
                && let Some(context) = slot.as_mut()
            {
                fact!(context, callbacks);
                if event.dwExtraInfo == PRODUCT_MARKER {
                    fact!(context, own_events);
                }
            }
        });
        if event.dwExtraInfo != PRODUCT_MARKER {
            let suppress = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                CONTEXT.with(|slot| {
                    let Ok(mut slot) = slot.try_borrow_mut() else {
                        return false;
                    };
                    slot.as_mut()
                        .is_some_and(|context| on_key(context, event, message.0 as u32))
                })
            }))
            .unwrap_or(false);
            #[cfg(debug_assertions)]
            CONTEXT.with(|slot| {
                if let Ok(mut slot) = slot.try_borrow_mut()
                    && let Some(context) = slot.as_mut()
                    && let Some(facts) = context.facts.as_mut()
                {
                    facts.callback_max_us = facts.callback_max_us.max(
                        callback_started
                            .elapsed()
                            .as_micros()
                            .min(u128::from(u64::MAX)) as u64,
                    );
                }
            });
            if suppress {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, message, data) }
}
unsafe extern "system" fn mouse_callback(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32
        && matches!(
            message.0 as u32,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_MOUSEWHEEL
        )
    {
        CONTEXT.with(|slot| {
            if let Ok(mut slot) = slot.try_borrow_mut()
                && let Some(context) = slot.as_mut()
            {
                context.history.reset();
                context.shared.generation.fetch_add(1, Ordering::SeqCst);
            }
        });
    }
    unsafe { CallNextHookEx(None, code, message, data) }
}
fn initial_keyboard() -> [u8; 256] {
    let mut keys = [0; 256];
    unsafe {
        if GetKeyboardState(&mut keys).is_err() {
            for key in [VK_CAPITAL, VK_NUMLOCK, VK_SCROLL] {
                keys[key.0 as usize] = (GetKeyState(i32::from(key.0)) & 1) as u8;
            }
        }
        seed_modifiers(&mut keys, |key| GetAsyncKeyState(key as i32) < 0);
    }
    keys
}
fn seed_modifiers(keys: &mut [u8; 256], mut down: impl FnMut(usize) -> bool) {
    for key in [0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0x5B, 0x5C] {
        update_keyboard(keys, key, down(key));
    }
}
fn update_keyboard(keys: &mut [u8; 256], vk: usize, down: bool) {
    if matches!(vk, 0x14 | 0x90 | 0x91) && down && keys[vk] & 0x80 == 0 {
        keys[vk] ^= 1;
    }
    keys[vk] = (keys[vk] & 1) | if down { 0x80 } else { 0 };
    for (generic, left, right) in [(0x10, 0xA0, 0xA1), (0x11, 0xA2, 0xA3), (0x12, 0xA4, 0xA5)] {
        if vk == left || vk == right {
            keys[generic] = (keys[left] | keys[right]) & 0x80;
        }
    }
}
fn on_key(context: &mut HookContext, event: KBDLLHOOKSTRUCT, message: u32) -> bool {
    if !context.shared.active.load(Ordering::SeqCst) {
        return false;
    }
    let down = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let up = matches!(message, WM_KEYUP | WM_SYSKEYUP);
    // Rejected input still changes the keyboard state seen by later physical keys.
    if event.vkCode <= 255 && (down || up) {
        update_keyboard(&mut context.keys, event.vkCode as usize, down);
    }
    let origin = input_origin(&event);
    if matches!(origin, InputOrigin::Software | InputOrigin::LowerIntegrity) {
        fact!(context, injected_events);
        if origin == InputOrigin::LowerIntegrity {
            fact!(context, lower_integrity_events);
        }
    }
    if !accepts_origin(origin, context.settings.allow_software_input) {
        context.history.reset();
        context.shared.generation.fetch_add(1, Ordering::SeqCst);
        return false;
    }
    if origin == InputOrigin::Software {
        fact!(context, software_accepted);
    }
    if event.vkCode > 255 {
        context.history.reset();
        return false;
    }
    if !down && !up {
        return false;
    }
    let vk = event.vkCode as usize;
    if down {
        fact!(context, key_down);
        if matches!(vk, 0x10..=0x12) {
            fact!(context, generic_modifiers);
        }
        if matches!(vk, 0xA0..=0xA5) {
            fact!(context, sided_modifiers);
        }
    }
    if up && context.swallowed_cancel == Some(vk as u16) {
        context.swallowed_cancel = None;
        return true;
    }
    if context.shared.busy.load(Ordering::SeqCst) {
        if down {
            context.shared.generation.fetch_add(1, Ordering::SeqCst);
        }
        context.history.reset();
        let cancel = down && context.settings.cancel_vk() == Some(vk as u16);
        if cancel {
            context.swallowed_cancel = Some(vk as u16);
        }
        return cancel;
    }
    if !down {
        return false;
    }
    if matches!(vk,0x10..=0x12|0xA0..=0xA5|0x5B|0x5C|0x14) {
        return false;
    }
    // Shortcut edits (notably Ctrl+Backspace) do not delete one scalar.
    if [0x11, 0x12, 0x5B, 0x5C]
        .iter()
        .any(|index| context.keys[*index] & 0x80 != 0)
    {
        fact!(context, shortcut_refused);
        context.history.reset();
        return false;
    }
    let generation = context.shared.generation.load(Ordering::SeqCst);
    let Ok(target) = current_stamp() else {
        fact!(context, stamp_refused);
        context.history.reset();
        return false;
    };
    if generation != context.generation || context.last_target != Some(target) {
        fact!(context, context_resets);
        context.history.reset();
        context.generation = generation;
        context.last_target = Some(target);
    }
    if target.process_id == unsafe { GetCurrentProcessId() } {
        fact!(context, own_target);
        context.history.reset();
        return false;
    }
    if vk == VK_BACK.0 as usize {
        return context
            .history
            .backspace()
            .is_some_and(|expansion| submit_match(context, expansion, target, generation));
    }
    if matches!(vk, 0x21..=0x28 | 0x2D | 0x2E | 0x1B) {
        context.history.reset();
        return false;
    }
    let mut units = [0u16; 8];
    let produced = unsafe {
        ToUnicodeEx(
            event.vkCode,
            event.scanCode,
            &context.keys,
            &mut units,
            4,
            Some(GetKeyboardLayout(target.thread_id)),
        )
    };
    if produced <= 0 || produced as usize > units.len() {
        fact!(context, translation_failed);
        context.history.reset();
        return false;
    }
    let Ok(text) = String::from_utf16(&units[..produced as usize]) else {
        context.history.reset();
        return false;
    };
    let Some(ch) = translated_character(&text) else {
        // A single suppressed key cannot be modeled as a partially delivered
        // multi-character composition. Invalidate instead of deleting guessed text.
        context.history.reset();
        return false;
    };
    fact!(context, translated_scalars);
    if ch == ' ' {
        fact!(context, spaces);
    }
    context
        .history
        .push(ch)
        .is_some_and(|expansion| submit_match(context, expansion, target, generation))
}
fn translated_character(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let ch = chars.next()?;
    chars
        .next()
        .is_none()
        .then_some(if ch == '\r' { '\n' } else { ch })
}
fn submit_match(
    context: &mut HookContext,
    expansion: Expansion,
    target: TargetStamp,
    generation: u64,
) -> bool {
    fact!(context, matches);
    if context
        .shared
        .busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return false;
    }
    let accepted = context
        .jobs
        .try_send(Job {
            expansion,
            target,
            ticket: generation,
        })
        .is_ok();
    if !accepted {
        context.shared.busy.store(false, Ordering::SeqCst);
    } else {
        fact!(context, accepted);
    }
    accepted
}
