//! Explicitly started observation thread. Constructing the app never installs hooks.
use super::{TargetStamp, current_stamp, unicode::OWN_INPUT_MARKER};
use crate::engine::{
    control::{DispatchGate, InputOrigin},
    recognizer::Recognizer,
    rules::RuleSet,
    runtime::ObservedCandidate,
    types::{EngineInput, PhysicalKey, RefusalReason},
};
use std::{
    cell::RefCell,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::{
            LibraryLoader::GetModuleHandleW,
            RemoteDesktop::{
                NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification,
                WTSUnRegisterSessionNotification,
            },
            Threading::GetCurrentThreadId,
        },
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
            Input::KeyboardAndMouse::{GetAsyncKeyState, GetKeyState, VK_CAPITAL},
            WindowsAndMessaging::*,
        },
    },
    core::w,
};

static ACTIVE_OBSERVER: AtomicBool = AtomicBool::new(false);

struct CloseOnExit(Arc<ObserverShared>);
impl Drop for CloseOnExit {
    fn drop(&mut self) {
        self.0.gate.close();
        self.0.halt.store(true, Ordering::SeqCst);
    }
}

#[derive(Clone, Copy)]
pub struct EligibleContext {
    pub stamp: TargetStamp,
    pub generation: u64,
}

pub struct ObserverShared {
    pub gate: Arc<DispatchGate>,
    pub eligible: Mutex<Option<EligibleContext>>,
    pub halt: AtomicBool,
    reason: AtomicU8,
    candidates: SyncSender<ObservedCandidate<TargetStamp>>,
    rules: Arc<RuleSet>,
}

impl ObserverShared {
    pub fn new(
        gate: Arc<DispatchGate>,
        candidates: SyncSender<ObservedCandidate<TargetStamp>>,
    ) -> Self {
        Self::with_rules(gate, candidates, Arc::new(RuleSet::fixtures()))
    }

    pub fn with_rules(
        gate: Arc<DispatchGate>,
        candidates: SyncSender<ObservedCandidate<TargetStamp>>,
        rules: Arc<RuleSet>,
    ) -> Self {
        Self {
            gate,
            eligible: Mutex::new(None),
            halt: AtomicBool::new(false),
            reason: AtomicU8::new(0),
            candidates,
            rules,
        }
    }
    pub fn take_reason(&self) -> Option<RefusalReason> {
        match self.reason.swap(0, Ordering::SeqCst) {
            1 => Some(RefusalReason::StaleContext),
            2 => Some(RefusalReason::InjectedInput),
            3 => Some(RefusalReason::QueueOverflow),
            4 => Some(RefusalReason::ContextUnavailable),
            _ => None,
        }
    }
    fn invalidate(&self, code: u8, require_rearm: bool) {
        self.reason.store(code, Ordering::SeqCst);
        if require_rearm {
            self.gate.close();
        } else {
            self.gate.invalidate_context();
        }
    }
}

struct HookState {
    keys: [bool; 256],
    caps: bool,
    generation: u64,
    recognizer: Recognizer,
}
thread_local! {
    static SHARED: RefCell<Option<Arc<ObserverShared>>> = const { RefCell::new(None) };
    static STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

fn with_shared(action: impl FnOnce(&Arc<ObserverShared>)) {
    SHARED.with(|slot| {
        if let Ok(shared) = slot.try_borrow()
            && let Some(shared) = shared.as_ref()
        {
            action(shared);
        }
    });
}

fn us_key(key: u32, shift: bool, caps: bool) -> PhysicalKey {
    if shift {
        return PhysicalKey::Other;
    }
    match key {
        0x41..=0x5A if !caps => PhysicalKey::Ascii((key - 0x41) as u8 + b'a'),
        0xBA => PhysicalKey::Ascii(b';'),
        0x20 => PhysicalKey::Space,
        _ => PhysicalKey::Other,
    }
}

fn input_origin(injected: bool, marker: usize) -> InputOrigin {
    if !injected {
        InputOrigin::Physical
    } else if marker == OWN_INPUT_MARKER {
        InputOrigin::Own
    } else {
        InputOrigin::UnexpectedInjected
    }
}

unsafe extern "system" fn keyboard(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: low-level hook provides this structure for the callback duration.
        let event = unsafe { *(data.0 as *const KBDLLHOOKSTRUCT) };
        let origin = input_origin(
            event.flags.contains(LLKHF_INJECTED) || event.flags.contains(LLKHF_LOWER_IL_INJECTED),
            event.dwExtraInfo,
        );
        with_shared(|shared| {
            shared.gate.observe(origin);
            if origin == InputOrigin::Own {
                return;
            }
            if origin == InputOrigin::UnexpectedInjected {
                shared.invalidate(2, false);
                return;
            }
            if !shared.gate.is_active() || shared.halt.load(Ordering::SeqCst) {
                return;
            }
            STATE.with(|slot| {
                let Ok(mut slot) = slot.try_borrow_mut() else {
                    shared.invalidate(1, false);
                    return;
                };
                let Some(state) = slot.as_mut() else {
                    return;
                };
                let down = matches!(message.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
                let up = matches!(message.0 as u32, WM_KEYUP | WM_SYSKEYUP);
                if !(down || up) || event.vkCode > 255 {
                    shared.invalidate(1, false);
                    return;
                }
                let repeat = down && state.keys[event.vkCode as usize];
                state.keys[event.vkCode as usize] = down;
                if event.vkCode == u32::from(VK_CAPITAL.0) && down && !repeat {
                    state.caps = !state.caps;
                }
                let ticket = shared.gate.snapshot();
                if state.generation != ticket.context {
                    state.recognizer.clear();
                    state.generation = ticket.context;
                }
                let eligible = match shared.eligible.try_lock() {
                    Ok(slot) => *slot,
                    Err(_) => None,
                };
                let Some(eligible) = eligible else {
                    state.recognizer.clear();
                    return;
                };
                if eligible.generation != ticket.context
                    || current_stamp().ok() != Some(eligible.stamp)
                {
                    state.recognizer.clear();
                    shared.invalidate(1, false);
                    return;
                }
                let modified = [
                    0x10, 0xA0, 0xA1, 0x11, 0xA2, 0xA3, 0x12, 0xA4, 0xA5, 0x5B, 0x5C,
                ]
                .iter()
                .any(|key| state.keys[*key]);
                let key = if modified {
                    PhysicalKey::Other
                } else {
                    us_key(event.vkCode, false, state.caps)
                };
                if key == PhysicalKey::Other {
                    state.recognizer.clear();
                    shared.invalidate(1, false);
                    return;
                }
                if let Some(candidate) =
                    state
                        .recognizer
                        .push(EngineInput::Key { key, down, repeat })
                {
                    let request = ObservedCandidate {
                        candidate,
                        origin: eligible.stamp,
                        ticket,
                        published_at: Instant::now(),
                    };
                    if shared.candidates.try_send(request).is_err() {
                        state.recognizer.clear();
                        shared.invalidate(3, false);
                    }
                }
            });
        });
    }
    // SAFETY: pass all events through; this probe never suppresses physical input.
    unsafe { CallNextHookEx(None, code, message, data) }
}

unsafe extern "system" fn pointer(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 && message.0 as u32 != WM_MOUSEMOVE {
        with_shared(|shared| shared.invalidate(1, false));
    }
    // SAFETY: pass the borrowed hook arguments onward unchanged.
    unsafe { CallNextHookEx(None, code, message, data) }
}

unsafe extern "system" fn focus(_: HWINEVENTHOOK, _: u32, _: HWND, _: i32, _: i32, _: u32, _: u32) {
    with_shared(|shared| shared.invalidate(1, false));
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_WTSSESSION_CHANGE
        || (message == WM_POWERBROADCAST && matches!(wparam.0, 4 | 7 | 18))
    {
        with_shared(|shared| {
            shared.invalidate(1, true);
            shared.halt.store(true, Ordering::SeqCst);
        });
        // SAFETY: this is the thread's own message loop.
        unsafe {
            PostQuitMessage(0);
        }
        return LRESULT(1);
    }
    // SAFETY: ordinary messages use Windows default handling for this window.
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

pub struct ObserverHandle {
    thread_id: u32,
    shared: Arc<ObserverShared>,
    done: Receiver<bool>,
    join: Option<JoinHandle<()>>,
    clean: Option<bool>,
}

impl ObserverHandle {
    pub fn start(shared: Arc<ObserverShared>) -> Result<Self, RefusalReason> {
        if ACTIVE_OBSERVER
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(RefusalReason::Busy);
        }
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        let state = shared.clone();
        let join = match thread::Builder::new()
            .name("texpio-observer".into())
            .spawn(move || {
                let clean = observe_thread(state, ready_tx);
                if clean {
                    ACTIVE_OBSERVER.store(false, Ordering::SeqCst);
                }
                let _ = done_tx.send(clean);
            }) {
            Ok(join) => join,
            Err(_) => {
                ACTIVE_OBSERVER.store(false, Ordering::SeqCst);
                return Err(RefusalReason::ContextUnavailable);
            }
        };
        match ready_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(thread_id)) => Ok(Self {
                thread_id,
                shared,
                done: done_rx,
                join: Some(join),
                clean: None,
            }),
            _ => {
                shared.halt.store(true, Ordering::SeqCst);
                shared.gate.close();
                // The startup thread observes halt and cleans itself; never join a
                // thread whose completion has not been acknowledged.
                if done_rx.recv_timeout(Duration::from_millis(500)).is_ok() {
                    let _ = join.join();
                }
                Err(RefusalReason::ContextUnavailable)
            }
        }
    }

    pub fn stop(&mut self) -> bool {
        if self.join.is_none() {
            return self.clean == Some(true);
        }
        self.shared.halt.store(true, Ordering::SeqCst);
        self.shared.gate.close();
        // SAFETY: readiness was published only after the message queue existed.
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        match self.done.recv_timeout(Duration::from_millis(500)) {
            Ok(clean) => {
                self.clean = Some(clean);
                if let Some(join) = self.join.take() {
                    let _ = join.join();
                }
                clean
            }
            Err(_) => false,
        }
    }
}

impl Drop for ObserverHandle {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[derive(Default)]
struct Resources {
    keyboard: Option<HHOOK>,
    pointer: Option<HHOOK>,
    events: Vec<HWINEVENTHOOK>,
    window: Option<HWND>,
    instance: Option<HINSTANCE>,
    registered: bool,
    session: bool,
}
impl Resources {
    fn close(&mut self) -> bool {
        let mut clean = true;
        // SAFETY: every resource was created on this observer thread and is closed
        // exactly once here; callback TLS stays alive until after cleanup.
        unsafe {
            for hook in [self.keyboard.take(), self.pointer.take()]
                .into_iter()
                .flatten()
            {
                clean &= UnhookWindowsHookEx(hook).is_ok();
            }
            for hook in self.events.drain(..) {
                clean &= UnhookWinEvent(hook).as_bool();
            }
            if let Some(window) = self.window.take() {
                if self.session {
                    clean &= WTSUnRegisterSessionNotification(window).is_ok();
                    self.session = false;
                }
                clean &= DestroyWindow(window).is_ok();
            }
            if self.registered {
                clean &= UnregisterClassW(w!("TexpioP0Observer"), self.instance).is_ok();
                self.registered = false;
            }
        }
        clean
    }
}
impl Drop for Resources {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn observe_thread(
    shared: Arc<ObserverShared>,
    ready: SyncSender<Result<u32, RefusalReason>>,
) -> bool {
    SHARED.with(|slot| *slot.borrow_mut() = Some(shared.clone()));
    let mut resources = Resources::default();
    let _close_on_exit = CloseOnExit(shared.clone());
    let result = (|| -> windows::core::Result<()> {
        // SAFETY: thread-affine native resources and valid initialized buffers.
        unsafe {
            let mut keys = [false; 256];
            for (key, held) in keys.iter_mut().enumerate() {
                *held = GetAsyncKeyState(key as i32) < 0;
            }
            STATE.with(|slot| {
                *slot.borrow_mut() = Some(HookState {
                    keys,
                    caps: GetKeyState(i32::from(VK_CAPITAL.0)) & 1 != 0,
                    generation: shared.gate.snapshot().context,
                    recognizer: Recognizer::from_rules(&shared.rules),
                })
            });
            let instance: HINSTANCE = GetModuleHandleW(None)?.into();
            resources.instance = Some(instance);
            let class = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: w!("TexpioP0Observer"),
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(windows::core::Error::from_thread());
            }
            resources.registered = true;
            let window = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("TexpioP0Observer"),
                w!("Texpio P0 observer"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                None,
                None,
                Some(instance),
                None,
            )?;
            resources.window = Some(window);
            WTSRegisterSessionNotification(window, NOTIFY_FOR_THIS_SESSION)?;
            resources.session = true;
            // A periodic wake also observes worker shutdown when the desktop is idle.
            // Destroying the window removes its timer during thread-affine cleanup.
            if SetTimer(Some(window), 1, 100, None) == 0 {
                return Err(windows::core::Error::from_thread());
            }
            if shared.halt.load(Ordering::SeqCst) {
                return Ok(());
            }
            resources.keyboard = Some(SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard),
                Some(instance),
                0,
            )?);
            resources.pointer = Some(SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(pointer),
                Some(instance),
                0,
            )?);
            for event in [EVENT_SYSTEM_FOREGROUND, EVENT_OBJECT_FOCUS] {
                let hook =
                    SetWinEventHook(event, event, None, Some(focus), 0, 0, WINEVENT_OUTOFCONTEXT);
                if hook.0.is_null() {
                    return Err(windows::core::Error::from_thread());
                }
                resources.events.push(hook);
            }
            let mut message = MSG::default();
            let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
            if shared.halt.load(Ordering::SeqCst) || ready.send(Ok(GetCurrentThreadId())).is_err() {
                return Ok(());
            }
            while !shared.halt.load(Ordering::SeqCst) {
                match GetMessageW(&mut message, None, 0, 0).0 {
                    -1 => return Err(windows::core::Error::from_thread()),
                    0 => break,
                    _ => {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
            Ok(())
        }
    })();
    shared.gate.close();
    if result.is_err() {
        shared.invalidate(4, true);
        let _ = ready.try_send(Err(RefusalReason::ContextUnavailable));
    }
    let clean = resources.close();
    STATE.with(|slot| *slot.borrow_mut() = None);
    SHARED.with(|slot| *slot.borrow_mut() = None);
    clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_mapping_is_explicit_and_rejects_modified_or_uppercase_input() {
        assert_eq!(us_key(0x41, false, false), PhysicalKey::Ascii(b'a'));
        assert_eq!(us_key(0xBA, false, true), PhysicalKey::Ascii(b';'));
        assert_eq!(us_key(0x41, false, true), PhysicalKey::Other);
        assert_eq!(us_key(0x41, true, false), PhysicalKey::Other);
        assert_eq!(us_key(0xBA, true, false), PhysicalKey::Other);
        assert_eq!(us_key(0x20, false, false), PhysicalKey::Space);
        assert_eq!(us_key(0x09, false, false), PhysicalKey::Other);
    }

    #[test]
    fn marker_only_identifies_injected_own_events() {
        assert_eq!(input_origin(false, OWN_INPUT_MARKER), InputOrigin::Physical);
        assert_eq!(input_origin(true, OWN_INPUT_MARKER), InputOrigin::Own);
        assert_eq!(input_origin(true, 0), InputOrigin::UnexpectedInjected);
    }

    #[test]
    fn absent_thread_handle_is_not_proof_of_successful_cleanup() {
        let (sender, _) = mpsc::sync_channel(1);
        let shared = Arc::new(ObserverShared::new(Arc::new(DispatchGate::new()), sender));
        let (_, done) = mpsc::sync_channel(1);
        let mut observer = ObserverHandle {
            thread_id: 0,
            shared,
            done,
            join: None,
            clean: Some(false),
        };
        assert!(!observer.stop());
    }
}
