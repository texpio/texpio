//! macOS session-tap product adapter. Native interaction requires separate qualification.
mod clipboard;
mod ffi;
mod policy;
use crate::product::{
    effects::{self, EffectBackend, ProductError},
    matcher::{Expansion, ProductRule, TypedHistory},
    service::{ProductPhase, Shared},
    settings::{InsertionMode, ProductSettings},
};
use ffi::*;
pub(crate) use policy::PermissionState;
use std::{
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const MARKER: i64 = 0x545850494f4d4143;
const MODIFIERS: u64 = (1 << 17) | (1 << 18) | (1 << 19) | (1 << 20);
static TAP_OWNED: AtomicBool = AtomicBool::new(false);
pub(crate) fn permission_state() -> PermissionState {
    unsafe {
        PermissionState {
            accessibility: AXIsProcessTrusted(),
            listen: CGPreflightListenEventAccess(),
            post: CGPreflightPostEventAccess(),
        }
    }
}
pub(crate) fn readiness() -> Result<(), ProductError> {
    if !permission_state().ready() || unsafe { IsSecureEventInputEnabled() } {
        return Err(ProductError::InputBlocked);
    }
    layout().map(drop).ok_or(ProductError::Unavailable)
}
/// The active input source must be an ASCII-capable keyboard layout, and its
/// `UCKeyboardLayout` data must be readable. Input methods and layouts that
/// cannot produce the ASCII triggers this profile accepts are refused here.
fn layout() -> Option<(String, Arc<Owned>)> {
    unsafe {
        let source = owned(TISCopyCurrentKeyboardInputSource())?;
        let id = text(TISGetInputSourceProperty(
            source.0,
            kTISPropertyInputSourceID,
        ))?;
        let kind = TISGetInputSourceProperty(source.0, kTISPropertyInputSourceType);
        let ascii = TISGetInputSourceProperty(source.0, kTISPropertyInputSourceIsASCIICapable);
        let is_keyboard_layout = !kind.is_null() && CFEqual(kind, kTISTypeKeyboardLayout);
        let ascii_capable = !ascii.is_null()
            && CFGetTypeID(ascii) == CFBooleanGetTypeID()
            && CFBooleanGetValue(ascii);
        if !policy::layout_supported(is_keyboard_layout, ascii_capable) {
            return None;
        }
        let data = TISGetInputSourceProperty(source.0, kTISPropertyUnicodeKeyLayoutData);
        if data.is_null()
            || CFGetTypeID(data) != CFDataGetTypeID()
            || CFDataGetBytePtr(data).is_null()
        {
            return None;
        }
        Some((id, Arc::new(source)))
    }
}
/// Convert one physical keydown with the current layout, carrying dead-key state
/// so a layout that composes accents cannot desynchronize history from the field.
fn translate(source: &Owned, code: u16, flags: u64, dead: &mut u32) -> Option<String> {
    const SHIFT: u64 = 1 << 17;
    const ALPHA_LOCK: u64 = 1 << 16;
    const CARBON_SHIFT: u32 = 0x0200;
    const CARBON_ALPHA_LOCK: u32 = 0x0400;
    let mut modifiers = 0;
    if flags & SHIFT != 0 {
        modifiers |= CARBON_SHIFT;
    }
    if flags & ALPHA_LOCK != 0 {
        modifiers |= CARBON_ALPHA_LOCK;
    }
    unsafe {
        let data = TISGetInputSourceProperty(source.0, kTISPropertyUnicodeKeyLayoutData);
        if data.is_null() || CFGetTypeID(data) != CFDataGetTypeID() {
            *dead = 0;
            return None;
        }
        let layout = CFDataGetBytePtr(data);
        if layout.is_null() {
            *dead = 0;
            return None;
        }
        let mut buffer = [0u16; 8];
        let mut length = 0;
        let status = UCKeyTranslate(
            layout.cast(),
            code,
            0, // kUCKeyActionDown
            modifiers,
            u32::from(LMGetKbdType()),
            0, // dead keys enabled: the returned state reports composition
            dead,
            buffer.len(),
            &mut length,
            buffer.as_mut_ptr(),
        );
        if status != 0 || length == 0 || length > buffer.len() {
            *dead = 0;
            return None;
        }
        String::from_utf16(&buffer[..length]).ok()
    }
}
/// Called only from the user's explicit Settings permission action.
pub(crate) fn request_permissions() -> PermissionState {
    let _pool = Pool::new();
    unsafe {
        let yes: Ref = send1(class(c"NSNumber"), c"numberWithBool:", true);
        let options: Ref = send2(
            class(c"NSDictionary"),
            c"dictionaryWithObject:forKey:",
            yes,
            kAXTrustedCheckOptionPrompt,
        );
        AXIsProcessTrustedWithOptions(options);
        // Accessibility trust normally satisfies listening and posting as well.
        // Ask for those only when macOS still withholds them from a trusted
        // process, so the Input Monitoring pane is not populated needlessly.
        if AXIsProcessTrusted() {
            if !CGPreflightListenEventAccess() {
                CGRequestListenEventAccess();
            }
            if !CGPreflightPostEventAccess() {
                CGRequestPostEventAccess();
            }
        }
    }
    permission_state()
}
#[derive(Clone)]
struct Target {
    pid: i32,
    field: Arc<Owned>,
    source: String,
    layout: Arc<Owned>,
    multiline: bool,
}
impl Target {
    fn same(&self, other: &Self) -> bool {
        self.pid == other.pid
            && self.source == other.source
            && self.multiline == other.multiline
            && unsafe { CFEqual(self.field.0, other.field.0) }
    }
}
fn target() -> Option<Target> {
    unsafe {
        if IsSecureEventInputEnabled() || !permission_state().ready() {
            return None;
        }
        let (source, layout) = layout()?;
        let system = owned(AXUIElementCreateSystemWide())?;
        if AXUIElementSetMessagingTimeout(system.0, 0.025) != 0 {
            return None;
        }
        let app = attribute(system.0, "AXFocusedApplication")?;
        if AXUIElementSetMessagingTimeout(app.0, 0.025) != 0 {
            return None;
        }
        let mut pid = 0;
        if AXUIElementGetPid(app.0, &mut pid) != 0 || pid == std::process::id() as i32 {
            return None;
        }
        let field = attribute(app.0, "AXFocusedUIElement")?;
        if AXUIElementSetMessagingTimeout(field.0, 0.025) != 0 {
            return None;
        }
        let role = attribute(field.0, "AXRole").and_then(|v| text(v.0))?;
        let subrole = attribute(field.0, "AXSubrole").and_then(|v| text(v.0));
        let marked = attribute(field.0, "AXMarkedText").and_then(|value| text(value.0));
        if !policy::context_evidence_safe(&role, subrole.as_deref(), marked.as_deref()) {
            return None;
        }
        // Require a normal editable text control, rather than a window identity alone.
        let range = string("AXSelectedTextRange");
        let mut settable = false;
        if AXUIElementIsAttributeSettable(field.0, range.0, &mut settable) != 0 || !settable {
            return None;
        }
        let editable = string("AXValue");
        if AXUIElementIsAttributeSettable(field.0, editable.0, &mut settable) != 0 || !settable {
            return None;
        }
        // Read range metadata only; do not copy the document's text through AX.
        #[repr(C)]
        struct Range {
            location: isize,
            length: isize,
        }
        let selected = attribute(field.0, "AXSelectedTextRange")?;
        let mut range = Range {
            location: -1,
            length: -1,
        };
        if CFGetTypeID(selected.0) != AXValueGetTypeID()
            || AXValueGetType(selected.0) != 4
            || !AXValueGetValue(selected.0, 4, (&mut range as *mut Range).cast())
            || range.location < 0
            || range.length != 0
        {
            return None;
        }
        Some(Target {
            pid,
            field: Arc::new(field),
            source,
            layout,
            multiline: role == "AXTextArea",
        })
    }
}
struct Snapshot {
    target: Option<Target>,
    updated: Instant,
    epoch: u64,
}
struct Job {
    expansion: Expansion,
    target: Target,
    generation: u64,
    admitted: Instant,
}
struct Capture {
    shared: Arc<Shared>,
    snapshot: Arc<Mutex<Snapshot>>,
    jobs: SyncSender<Job>,
    history: TypedHistory,
    epoch: u64,
    generation: u64,
    cancel: u16,
    allow_software: bool,
    paste: bool,
    restore: bool,
    dead: u32,
}
impl Capture {
    /// History only mirrors the field while one keydown equals one character.
    fn reset(&mut self) {
        self.history.reset();
        self.dead = 0;
    }
}
fn cancel_code(value: &str) -> Option<u16> {
    match value {
        "Escape" => Some(53),
        "F1" => Some(122),
        "F2" => Some(120),
        "F3" => Some(99),
        "F4" => Some(118),
        "F5" => Some(96),
        "F6" => Some(97),
        "F7" => Some(98),
        "F8" => Some(100),
        "F9" => Some(101),
        "F10" => Some(109),
        "F11" => Some(103),
        "F12" => Some(111),
        _ => None,
    }
}
unsafe extern "C" fn callback(_: Ref, kind: u32, event: Event, user: Ref) -> Event {
    // Do not unwind across Quartz's C callback boundary.
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        capture(kind, event, &mut *(user as *mut Capture))
    }))
    .unwrap_or(event)
}
unsafe fn capture(kind: u32, event: Event, context: &mut Capture) -> Event {
    unsafe {
        if kind == 0xffff_fffe || kind == 0xffff_ffff {
            context.reset();
            context.shared.generation.fetch_add(1, Ordering::SeqCst);
            context.shared.active.store(false, Ordering::SeqCst);
            if let Ok(mut status) = context.shared.status.try_lock() {
                status.phase = ProductPhase::Fault;
                status.last_error = Some(ProductError::InputBlocked);
            }
            return event;
        }
        if CGEventGetIntegerValueField(event, 42) == MARKER {
            return event;
        }
        if !context.shared.active.load(Ordering::SeqCst) {
            context.reset();
            return event;
        }
        let generation = context.shared.generation.load(Ordering::SeqCst);
        if context.generation != generation {
            context.reset();
            context.generation = generation;
        }
        if kind != 10 {
            if kind != 11 && kind != 12 {
                context.reset();
                context.shared.generation.fetch_add(1, Ordering::SeqCst);
            }
            return event;
        }
        let code = CGEventGetIntegerValueField(event, 9) as u16;
        if code == context.cancel || context.shared.busy.load(Ordering::SeqCst) {
            context.reset();
            context.shared.generation.fetch_add(1, Ordering::SeqCst);
            return event;
        }
        if CGEventGetFlags(event) & ((1 << 18) | (1 << 19) | (1 << 20)) != 0
            || (!context.allow_software && CGEventGetIntegerValueField(event, 41) != 0)
        {
            context.reset();
            return event;
        }
        let snapshot = match context.snapshot.try_lock() {
            Ok(value) => value,
            Err(_) => {
                context.history.reset();
                context.dead = 0;
                return event;
            }
        };
        if snapshot.epoch != context.epoch {
            context.history.reset();
            context.dead = 0;
            context.epoch = snapshot.epoch;
        }
        let Some(target) = snapshot
            .target
            .as_ref()
            .filter(|_| snapshot.updated.elapsed() < Duration::from_millis(100))
            .cloned()
        else {
            context.history.reset();
            context.dead = 0;
            return event;
        };
        drop(snapshot);
        let event_pid = CGEventGetIntegerValueField(event, 40);
        if event_pid > 0 && event_pid != i64::from(target.pid) {
            context.reset();
            return event;
        }
        let expansion = if code == 51 {
            context.dead = 0;
            context.history.backspace()
        } else {
            // Return/Tab are deliberately excluded from the initial native profile.
            if matches!(code, 36 | 48 | 76) {
                context.reset();
                return event;
            }
            // A dead-key press, or the keystroke that completes one, delivers
            // fewer characters than keys. History cannot track that, so both
            // refuse rather than desynchronizing the deletion count.
            let composing = context.dead != 0;
            let character = translate(
                &target.layout,
                code,
                CGEventGetFlags(event),
                &mut context.dead,
            );
            if composing || context.dead != 0 {
                context.reset();
                return event;
            }
            let Some(character) = character.as_deref().and_then(policy::accepted_scalar) else {
                context.reset();
                return event;
            };
            context.history.push(character)
        };
        let Some(expansion) = expansion else {
            return event;
        };
        if !policy::admission_supported(
            &expansion.rule.body,
            target.multiline,
            context.paste,
            context.restore,
        ) {
            context.reset();
            return event;
        }
        let accepted = policy::admit(
            &context.shared.busy,
            &context.jobs,
            Job {
                expansion,
                target,
                generation,
                admitted: Instant::now(),
            },
        );
        context.history.reset();
        if accepted { ptr::null_mut() } else { event }
    }
}
pub(crate) struct Runtime {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    done: mpsc::Receiver<()>,
    shared: Arc<Shared>,
}
impl Runtime {
    pub(crate) fn launch(
        shared: Arc<Shared>,
        rules: Vec<ProductRule>,
        settings: ProductSettings,
    ) -> Result<Self, ProductError> {
        if !policy::mode_supported(
            settings.insertion == InsertionMode::Paste,
            settings.restore_clipboard,
        ) {
            return Err(ProductError::InvalidSettings);
        }
        readiness()?;
        let cancel = cancel_code(&settings.cancel_key).ok_or(ProductError::InvalidSettings)?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = stop.clone();
        let native_shared = shared.clone();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let (done_tx, done) = mpsc::sync_channel(1);
        // Non-ASCII triggers remain stored, but cannot enter this deletion profile.
        let resolver = shared.files.lock().map_err(|_| ProductError::Busy)?.clone();
        let rules = rules
            .into_iter()
            .filter(|rule| {
                rule.trigger.is_ascii()
                    && rule.rich_html.is_none()
                    && policy::payload_supported(&rule.body, true)
                    && resolver.as_ref().is_none_or(|resolve| {
                        resolve(&rule.id).is_ok_and(|files| files.files.is_empty())
                    })
            })
            .collect();
        if TAP_OWNED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(ProductError::Busy);
        }
        let spawned = thread::Builder::new()
            .name("texpio-macos-tap".into())
            .spawn(move || {
                let _pool = Pool::new();
                let result = run(
                    native_shared.clone(),
                    rules,
                    settings,
                    cancel,
                    worker_stop,
                    &ready_tx,
                );
                if let Err(error) = result {
                    let _ = ready_tx.try_send(Err(error));
                    native_shared.active.store(false, Ordering::SeqCst);
                }
                TAP_OWNED.store(false, Ordering::SeqCst);
                let _ = done_tx.send(());
            });
        let thread = match spawned {
            Ok(thread) => thread,
            Err(_) => {
                TAP_OWNED.store(false, Ordering::SeqCst);
                return Err(ProductError::Unavailable);
            }
        };
        let mut runtime = Self {
            stop,
            thread: Some(thread),
            done,
            shared,
        };
        match ready_rx.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(())) => Ok(runtime),
            Ok(Err(error)) => {
                runtime.shutdown();
                Err(error)
            }
            Err(_) => {
                runtime.shutdown();
                Err(ProductError::Unavailable)
            }
        }
    }
    pub(crate) fn shutdown(&mut self) -> bool {
        self.stop.store(true, Ordering::SeqCst);
        self.shared.generation.fetch_add(1, Ordering::SeqCst);
        if self.thread.is_none() {
            return true;
        }
        if self.done.recv_timeout(Duration::from_secs(2)).is_err() {
            return false;
        }
        self.thread
            .take()
            .is_none_or(|thread| thread.join().is_ok())
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
fn run(
    shared: Arc<Shared>,
    rules: Vec<ProductRule>,
    settings: ProductSettings,
    cancel: u16,
    stop: Arc<AtomicBool>,
    ready: &SyncSender<Result<(), ProductError>>,
) -> Result<(), ProductError> {
    let snapshot = Arc::new(Mutex::new(Snapshot {
        target: None,
        updated: Instant::now(),
        epoch: 0,
    }));
    let (jobs, receiver) = mpsc::sync_channel::<Job>(1);
    let mut context = Box::new(Capture {
        shared: shared.clone(),
        snapshot: snapshot.clone(),
        jobs,
        history: TypedHistory::new(rules, settings.boundary, settings.activation),
        epoch: 0,
        generation: shared.generation.load(Ordering::SeqCst),
        cancel,
        allow_software: settings.allow_software_input,
        paste: settings.insertion == InsertionMode::Paste,
        restore: settings.restore_clipboard,
        dead: 0,
    });
    let mask = [1u32, 3, 5, 10, 11, 12, 22, 25, 26, 27]
        .into_iter()
        .fold(0, |mask, kind| mask | (1u64 << kind));
    let tap = Tap(owned(unsafe {
        CGEventTapCreate(
            1,
            0,
            0,
            mask,
            callback,
            (&mut *context as *mut Capture).cast(),
        )
    })
    .ok_or(ProductError::InputBlocked)?);
    let source = owned(unsafe { CFMachPortCreateRunLoopSource(ptr::null_mut(), tap.0.0, 0) })
        .ok_or(ProductError::Unavailable)?;
    let effect_shared = shared.clone();
    let effect_stop = stop.clone();
    let worker = thread::Builder::new()
        .name("texpio-macos-effects".into())
        .spawn(move || {
            while !effect_stop.load(Ordering::SeqCst) {
                let Ok(job) = receiver.recv_timeout(Duration::from_millis(20)) else {
                    continue;
                };
                let _pool = Pool::new();
                let result = effect_shared
                    .effects
                    .lock()
                    .map_err(|_| ProductError::Busy)
                    .and_then(|_guard| {
                        let resolver = effect_shared
                            .files
                            .lock()
                            .map_err(|_| ProductError::Busy)?
                            .clone();
                        if let Some(resolve) = resolver
                            && !resolve(&job.expansion.rule.id)?.files.is_empty()
                        {
                            return Err(ProductError::Unavailable);
                        }
                        let mut backend = NativeEdit {
                            shared: effect_shared.clone(),
                            target: job.target,
                            generation: job.generation,
                            settings: settings.clone(),
                            lease: None,
                            submitted: false,
                            deadline: job.admitted + Duration::from_secs(30),
                        };
                        effects::wait_for_modifier_release(
                            || backend.check(),
                            || thread::sleep(Duration::from_millis(10)),
                        )?;
                        effects::perform(&mut backend, &job.expansion, &settings)
                    });
                if let Ok(mut status) = effect_shared.status.lock() {
                    status.last_error = result.err();
                    if matches!(result, Err(ProductError::Cancelled)) {
                        status.cancelled = status.cancelled.saturating_add(1);
                    }
                }
                if result.is_ok()
                    && let Ok(recorder) = effect_shared.used.lock()
                    && let Some(recorder) = recorder.as_ref()
                {
                    recorder(&job.expansion.rule.id);
                }
                effect_shared.busy.store(false, Ordering::SeqCst);
            }
        })
        .map_err(|_| ProductError::Unavailable)?;
    let context_stop = stop.clone();
    let context_shared = shared.clone();
    let context_worker = match thread::Builder::new()
        .name("texpio-macos-context".into())
        .spawn(move || {
            let mut last_poll = Instant::now();
            while !context_stop.load(Ordering::SeqCst) {
                let gap = last_poll.elapsed() > Duration::from_millis(250);
                last_poll = Instant::now();
                let current = target();
                if let Ok(mut snapshot) = snapshot.lock() {
                    let same = match (&snapshot.target, &current) {
                        (Some(a), Some(b)) => a.same(b),
                        (None, None) => true,
                        _ => false,
                    };
                    if !same || gap {
                        snapshot.epoch = snapshot.epoch.wrapping_add(1);
                        context_shared.generation.fetch_add(1, Ordering::SeqCst);
                    }
                    snapshot.target = current;
                    snapshot.updated = Instant::now();
                }
                thread::sleep(Duration::from_millis(10));
            }
        }) {
        Ok(worker) => worker,
        Err(_) => {
            stop.store(true, Ordering::SeqCst);
            let _ = worker.join();
            return Err(ProductError::Unavailable);
        }
    };
    unsafe {
        CFRunLoopAddSource(CFRunLoopGetCurrent(), source.0, kCFRunLoopDefaultMode);
        CGEventTapEnable(tap.0.0, true);
    }
    let _ = ready.send(Ok(()));
    while !stop.load(Ordering::SeqCst) && shared.active.load(Ordering::SeqCst) {
        if !permission_state().ready()
            || !unsafe { CGEventTapIsEnabled(tap.0.0) }
            || worker.is_finished()
            || context_worker.is_finished()
        {
            shared.active.store(false, Ordering::SeqCst);
            if let Ok(mut status) = shared.status.lock() {
                status.phase = ProductPhase::Fault;
                status.last_error = Some(ProductError::InputBlocked);
            }
            break;
        }
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.01, false);
        }
    }
    stop.store(true, Ordering::SeqCst);
    shared.generation.fetch_add(1, Ordering::SeqCst);
    unsafe {
        CGEventTapEnable(tap.0.0, false);
        CFMachPortInvalidate(tap.0.0);
        CFRunLoopRemoveSource(CFRunLoopGetCurrent(), source.0, kCFRunLoopDefaultMode);
    }
    drop(context);
    let _ = worker.join();
    let _ = context_worker.join();
    shared.busy.store(false, Ordering::SeqCst);
    Ok(())
}
struct NativeEdit {
    shared: Arc<Shared>,
    target: Target,
    generation: u64,
    settings: ProductSettings,
    lease: Option<clipboard::Lease>,
    submitted: bool,
    deadline: Instant,
}
impl NativeEdit {
    fn key(&mut self, code: u16, flags: u64, unicode: Option<&[u16]>) -> Result<(), ProductError> {
        self.check()?;
        unsafe {
            // Allocate the pair before submitting either event; never strand a modifier.
            let down = owned(CGEventCreateKeyboardEvent(ptr::null_mut(), code, true))
                .ok_or(ProductError::InputBlocked)?;
            let up = owned(CGEventCreateKeyboardEvent(ptr::null_mut(), code, false))
                .ok_or(ProductError::InputBlocked)?;
            for event in [&down, &up] {
                CGEventSetIntegerValueField(event.0, 42, MARKER);
                CGEventSetFlags(event.0, flags);
                if let Some(text) = unicode {
                    CGEventKeyboardSetUnicodeString(event.0, text.len(), text.as_ptr());
                }
            }
            CGEventPost(1, down.0);
            CGEventPost(1, up.0);
        }
        if !self.submitted {
            self.submitted = true;
            if let Ok(mut status) = self.shared.status.lock() {
                status.submitted = status.submitted.saturating_add(1);
            }
        }
        thread::sleep(Duration::from_secs_f64(
            2.0 / self.settings.typing_speed.events_per_second() as f64,
        ));
        Ok(())
    }
}
impl EffectBackend for NativeEdit {
    fn check(&mut self) -> Result<(), ProductError> {
        if Instant::now() >= self.deadline
            || !self.shared.active.load(Ordering::SeqCst)
            || self.shared.generation.load(Ordering::SeqCst) != self.generation
        {
            return Err(ProductError::Cancelled);
        }
        let current = target().ok_or(ProductError::InputBlocked)?;
        if !policy::context_valid(
            true,
            self.shared.generation.load(Ordering::SeqCst),
            self.generation,
            self.target.same(&current),
            unsafe { IsSecureEventInputEnabled() },
        ) {
            return Err(ProductError::TargetChanged);
        }
        if unsafe { CGEventSourceFlagsState(1) } & MODIFIERS != 0 {
            return Err(ProductError::ModifiersHeld);
        }
        if self.lease.as_ref().is_some_and(|lease| !lease.owns()) {
            return Err(ProductError::Clipboard);
        }
        Ok(())
    }
    fn prepare(&mut self, text: &str, mode: InsertionMode) -> Result<(), ProductError> {
        if !policy::admission_supported(
            text,
            self.target.multiline,
            mode == InsertionMode::Paste,
            self.settings.restore_clipboard,
        ) {
            return Err(ProductError::InputBlocked);
        }
        self.check()?;
        if mode == InsertionMode::Paste && !text.is_empty() {
            let restore = self.settings.restore_clipboard;
            self.lease = Some(clipboard::Lease::acquire(text, restore, || self.check())?);
        }
        self.check()
    }
    fn erase(&mut self, count: usize) -> Result<(), ProductError> {
        for _ in 0..count {
            self.key(51, 0, None)?;
        }
        Ok(())
    }
    fn content(&mut self, text: &str, mode: InsertionMode) -> Result<(), ProductError> {
        match mode {
            InsertionMode::Type => {
                for scalar in policy::unicode_events(text) {
                    if scalar == [10] {
                        self.key(36, 0, None)?;
                    } else {
                        self.key(0, 0, Some(&scalar))?;
                    }
                }
            }
            InsertionMode::Paste => {
                if self.lease.is_none() {
                    return Err(ProductError::Clipboard);
                }
                self.key(9, 1 << 20, None)?;
                for _ in 0..15 {
                    thread::sleep(Duration::from_millis(10));
                    self.check()?;
                }
            }
        }
        Ok(())
    }
    fn after_content(&mut self) -> Result<(), ProductError> {
        self.check()?;
        self.lease.take();
        Ok(())
    }
    fn terminator(&mut self, character: char) -> Result<(), ProductError> {
        if !character.is_ascii() || character.is_control() {
            return Err(ProductError::InputBlocked);
        }
        let mut buf = [0; 2];
        self.key(0, 0, Some(character.encode_utf16(&mut buf)))
    }
}
