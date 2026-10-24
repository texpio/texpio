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
                let history = TypedHistory::new(rules, settings.boundary);
                CONTEXT.with(|slot| {
                    *slot.borrow_mut() = Some(HookContext {
                        shared: hook_shared.clone(),
                        jobs,
                        history,
                        settings,
                        keys: [0; 256],
                        last_target: None,
                        generation,
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
                CONTEXT.with(|slot| *slot.borrow_mut() = None);
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
    if code == HC_ACTION as i32 && data.0 != 0 {
        // SAFETY: hook owns the borrowed event for this callback only.
        let event = unsafe { *(data.0 as *const KBDLLHOOKSTRUCT) };
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
fn on_key(context: &mut HookContext, event: KBDLLHOOKSTRUCT, message: u32) -> bool {
    if !context.shared.active.load(Ordering::SeqCst) {
        return false;
    }
    if event.flags.contains(LLKHF_INJECTED) || event.flags.contains(LLKHF_LOWER_IL_INJECTED) {
        context.history.reset();
        context.shared.generation.fetch_add(1, Ordering::SeqCst);
        return false;
    }
    if event.vkCode > 255 {
        context.history.reset();
        return false;
    }
    let down = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let up = matches!(message, WM_KEYUP | WM_SYSKEYUP);
    if !down && !up {
        return false;
    }
    let vk = event.vkCode as usize;
    if vk == VK_CAPITAL.0 as usize && down {
        context.keys[vk] ^= 1;
    }
    context.keys[vk] = (context.keys[vk] & 1) | if down { 0x80 } else { 0 };
    for (generic, left, right) in [(0x10, 0xA0, 0xA1), (0x11, 0xA2, 0xA3), (0x12, 0xA4, 0xA5)] {
        context.keys[generic] = (context.keys[left] | context.keys[right]) & 0x80;
    }
    if context.shared.busy.load(Ordering::SeqCst) {
        if down {
            context.shared.generation.fetch_add(1, Ordering::SeqCst);
        }
        context.history.reset();
        return down && context.settings.cancel_vk() == Some(vk as u16);
    }
    if !down {
        return false;
    }
    let generation = context.shared.generation.load(Ordering::SeqCst);
    let Ok(target) = current_stamp() else {
        context.history.reset();
        return false;
    };
    if generation != context.generation || context.last_target != Some(target) {
        context.history.reset();
        context.generation = generation;
        context.last_target = Some(target);
    }
    if target.process_id == unsafe { GetCurrentProcessId() } {
        context.history.reset();
        return false;
    }
    if vk == VK_BACK.0 as usize {
        context.history.backspace();
        return false;
    }
    if matches!(vk, 0x21..=0x28 | 0x2D | 0x2E | 0x1B) {
        context.history.reset();
        return false;
    }
    if matches!(vk,0x10..=0x12|0xA0..=0xA5|0x5B|0x5C|0x14) {
        return false;
    }
    if [0x11, 0x12, 0x5B, 0x5C]
        .iter()
        .any(|index| context.keys[*index] & 0x80 != 0)
    {
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
    if produced <= 0 {
        context.history.reset();
        return false;
    }
    let Ok(text) = String::from_utf16(&units[..produced as usize]) else {
        context.history.reset();
        return false;
    };
    for mut ch in text.chars() {
        if ch == '\r' {
            ch = '\n';
        }
        if let Some(expansion) = context.history.push(ch) {
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
            }
            return accepted;
        }
    }
    false
}
