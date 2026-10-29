//! Native X11 observation and bounded serialized effects. This module never grabs or
//! rewrites the keyboard and never treats an XWayland connection as native Wayland.
mod clipboard;
mod context;
mod keyboard;
mod policy;

use crate::product::{
    effects::{self, EffectBackend, ProductError},
    matcher::{Expansion, ProductRule, TypedHistory},
    service::Shared,
    settings::{InsertionMode, ProductSettings},
};
#[cfg(unix)]
use std::os::fd::AsRawFd;
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{
        Event,
        record::{self, ConnectionExt as _},
        xfixes::ConnectionExt as _,
        xkb::{self, ConnectionExt as _},
        xproto::*,
        xtest::ConnectionExt as _,
    },
    rust_connection::RustConnection,
    x11_utils::{Serialize, TryParse},
};

fn fail<T>(_: T) -> ProductError {
    ProductError::Unavailable
}
const CORE: u16 = 0x100;
type Expected = Arc<Mutex<VecDeque<(u8, u8)>>>;
type Connections = Arc<Mutex<Vec<Arc<RustConnection>>>>;

fn connect(connections: &Connections) -> Result<(Arc<RustConnection>, usize), ProductError> {
    let (connection, screen) = x11rb::connect(None).map_err(fail)?;
    let connection = Arc::new(connection);
    connections.lock().map_err(fail)?.push(connection.clone());
    Ok((connection, screen))
}
fn interrupt(connections: &Connections) {
    if let Ok(connections) = connections.lock() {
        for connection in connections.iter() {
            #[cfg(unix)]
            // SAFETY: shutdown borrows a live fd, and Arc ownership prevents reuse.
            unsafe {
                libc::shutdown(connection.stream().as_raw_fd(), libc::SHUT_RDWR);
            }
            #[cfg(not(unix))]
            let _ = connection;
        }
    }
}
fn prerequisites(connection: &RustConnection) -> Result<(), ProductError> {
    for name in ["RECORD", "XKEYBOARD", "XTEST", "XFIXES"] {
        if connection
            .extension_information(name)
            .map_err(fail)?
            .is_none()
        {
            return Err(ProductError::Unavailable);
        }
    }
    connection
        .record_query_version(1, 13)
        .map_err(fail)?
        .reply()
        .map_err(fail)?;
    if !connection
        .xkb_use_extension(1, 0)
        .map_err(fail)?
        .reply()
        .map_err(fail)?
        .supported
    {
        return Err(ProductError::Unavailable);
    }
    connection
        .xtest_get_version(2, 2)
        .map_err(fail)?
        .reply()
        .map_err(fail)?;
    connection
        .xfixes_query_version(5, 0)
        .map_err(fail)?
        .reply()
        .map_err(fail)?;
    Ok(())
}
fn session_ready() -> Result<(), ProductError> {
    if !policy::native_x11(
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var_os("WAYLAND_DISPLAY").is_some(),
    ) {
        return Err(ProductError::Unavailable);
    }
    // XIM/IBus/Fcitx composition is outside the initial positive ASCII profile.
    for variable in ["GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS"] {
        if let Ok(value) = std::env::var(variable)
            && !value.is_empty()
            && !["none", "@im=none", "xim"].contains(&value.as_str())
        {
            return Err(ProductError::Unavailable);
        }
    }
    Ok(())
}
pub(crate) fn readiness() -> Result<(), ProductError> {
    session_ready()?;
    let (connection, _) = x11rb::connect(None).map_err(fail)?;
    prerequisites(&connection)?;
    let _ = target(&connection)?;
    context::provider_ready()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Target {
    focus: Window,
    group: u8,
    map: u64,
    pid: u32,
}
fn target(connection: &RustConnection) -> Result<Target, ProductError> {
    let focus = connection
        .get_input_focus()
        .map_err(fail)?
        .reply()
        .map_err(fail)?
        .focus;
    if focus <= 1 {
        return Err(ProductError::TargetChanged);
    }
    let state = connection
        .xkb_get_state(CORE)
        .map_err(fail)?
        .reply()
        .map_err(fail)?;
    // Include key types, virtual modifiers, actions and behaviors, not only core
    // keysyms: a type-only XKB update can change text without changing symbols.
    let mut map = connection
        .xkb_get_map(
            CORE,
            xkb::MapPart::from(0xff_u16),
            xkb::MapPart::default(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            xkb::VMod::default(),
            0,
            0,
            0,
            0,
            0,
            0,
        )
        .map_err(fail)?
        .reply()
        .map_err(fail)?;
    map.sequence = 0;
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    map.serialize().hash(&mut hash);
    Ok(Target {
        focus,
        group: u8::from(state.group),
        map: hash.finish(),
        pid: window_pid(connection, focus)?,
    })
}
fn window_pid(connection: &RustConnection, mut window: Window) -> Result<u32, ProductError> {
    let pid = connection
        .intern_atom(false, b"_NET_WM_PID")
        .map_err(fail)?
        .reply()
        .map_err(fail)?
        .atom;
    for _ in 0..32 {
        let value = connection
            .get_property(false, window, pid, AtomEnum::CARDINAL, 0, 1)
            .map_err(fail)?
            .reply()
            .map_err(fail)?;
        if let Some(pid) = value
            .value32()
            .and_then(|mut values| values.next())
            .filter(|pid| *pid != 0)
        {
            return Ok(pid);
        }
        let parent = connection
            .query_tree(window)
            .map_err(fail)?
            .reply()
            .map_err(fail)?;
        if parent.parent == 0 || parent.parent == parent.root {
            break;
        }
        window = parent.parent;
    }
    Err(ProductError::InputBlocked)
}

pub(crate) struct Runtime {
    shared: Arc<Shared>,
    stopped: Arc<AtomicBool>,
    connections: Connections,
    done: Receiver<()>,
    thread: Option<JoinHandle<()>>,
}
impl Runtime {
    pub(crate) fn launch(
        shared: Arc<Shared>,
        rules: Vec<ProductRule>,
        settings: ProductSettings,
    ) -> Result<Self, ProductError> {
        readiness()?;
        // Unqualified Unicode triggers must never authorize scalar-count deletion.
        let rules: Vec<_> = rules
            .into_iter()
            .filter(|rule| rule.trigger.is_ascii())
            .collect();
        let stopped = Arc::new(AtomicBool::new(false));
        let connections = Arc::new(Mutex::new(Vec::new()));
        let (ready_tx, ready) = mpsc::sync_channel(1);
        let (done_tx, done) = mpsc::sync_channel(1);
        let runtime_shared = shared.clone();
        let runtime_stop = stopped.clone();
        let runtime_connections = connections.clone();
        let thread = thread::Builder::new()
            .name("texpio-x11-runtime".into())
            .spawn(move || {
                let result = run(
                    runtime_shared.clone(),
                    runtime_stop,
                    runtime_connections,
                    rules,
                    settings,
                    &ready_tx,
                );
                if let Err(error) = result {
                    runtime_shared.active.store(false, Ordering::SeqCst);
                    runtime_shared.generation.fetch_add(1, Ordering::SeqCst);
                    if let Ok(mut status) = runtime_shared.status.lock() {
                        status.phase = crate::product::service::ProductPhase::Fault;
                        status.last_error = Some(error);
                    }
                    let _ = ready_tx.try_send(Err(error));
                }
                runtime_shared.busy.store(false, Ordering::SeqCst);
                let _ = done_tx.send(());
            })
            .map_err(fail)?;
        let mut runtime = Self {
            shared,
            stopped,
            connections,
            done,
            thread: Some(thread),
        };
        match ready.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => Ok(runtime),
            _ => {
                runtime.shutdown();
                Err(ProductError::Unavailable)
            }
        }
    }
    pub(crate) fn shutdown(&mut self) -> bool {
        self.stopped.store(true, Ordering::SeqCst);
        self.shared.active.store(false, Ordering::SeqCst);
        self.shared.generation.fetch_add(1, Ordering::SeqCst);
        // Socket shutdown interrupts blocked RECORD/reply reads. Never close a borrowed fd:
        // the Arc owner closes it, preventing descriptor-reuse races.
        interrupt(&self.connections);
        if self.thread.is_some()
            && self.done.recv_timeout(Duration::from_secs(2)).is_ok()
            && let Some(thread) = self.thread.take()
        {
            let _ = thread.join();
        }
        self.thread.is_none()
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct Job {
    expansion: Expansion,
    target: Target,
    generation: u64,
    field: context::Evidence,
}
fn invalidate(shared: &Shared, history: &mut TypedHistory) {
    history.reset();
    shared.generation.fetch_add(1, Ordering::SeqCst);
}
fn run(
    shared: Arc<Shared>,
    stopped: Arc<AtomicBool>,
    connections: Connections,
    rules: Vec<ProductRule>,
    settings: ProductSettings,
    ready: &mpsc::SyncSender<Result<(), ProductError>>,
) -> Result<(), ProductError> {
    let (control, _) = connect(&connections)?;
    prerequisites(&control)?;
    let (recording, _) = connect(&connections)?;
    let (effects, screen) = connect(&connections)?;
    let mut keyboard = keyboard::Keyboard::open()?;
    let field_context = context::Guard::start()?;
    control
        .xkb_select_events(
            CORE,
            xkb::EventType::default(),
            xkb::EventType::NEW_KEYBOARD_NOTIFY
                | xkb::EventType::MAP_NOTIFY
                | xkb::EventType::STATE_NOTIFY,
            xkb::MapPart::KEY_TYPES | xkb::MapPart::KEY_SYMS,
            xkb::MapPart::KEY_TYPES | xkb::MapPart::KEY_SYMS,
            &xkb::SelectEventsAux::default(),
        )
        .map_err(fail)?
        .check()
        .map_err(fail)?;
    let mut clipboard = clipboard::Clipboard::new(effects.clone(), screen)?;
    let context = control.generate_id().map_err(fail)?;
    let xtest_opcode = control
        .extension_information("XTEST")
        .map_err(fail)?
        .ok_or(ProductError::Unavailable)?
        .major_opcode;
    let effects_base = effects.setup().resource_id_base;
    control
        .record_create_context(
            context,
            record::ElementHeader::default(),
            &[3],
            &[record::Range {
                device_events: record::Range8 {
                    first: KEY_PRESS_EVENT,
                    last: MOTION_NOTIFY_EVENT,
                },
                ext_requests: record::ExtRange {
                    major: record::Range8 {
                        first: xtest_opcode,
                        last: xtest_opcode,
                    },
                    minor: record::Range16 { first: 2, last: 2 },
                },
                delivered_events: record::Range8 {
                    first: FOCUS_IN_EVENT,
                    last: FOCUS_OUT_EVENT,
                },
                ..Default::default()
            }],
        )
        .map_err(fail)?
        .check()
        .map_err(fail)?;
    let expected: Expected = Arc::new(Mutex::new(VecDeque::new()));
    let (jobs, receiver) = mpsc::sync_channel::<Job>(1);
    let worker_shared = shared.clone();
    let worker_stop = stopped.clone();
    let worker_expected = expected.clone();
    let worker_connections = connections.clone();
    let worker_context = field_context.clone();
    let worker_settings = settings.clone();
    let worker = thread::Builder::new()
        .name("texpio-x11-effects".into())
        .spawn(move || {
            while !worker_stop.load(Ordering::SeqCst) && worker_shared.active.load(Ordering::SeqCst)
            {
                if clipboard.pump().is_err() {
                    break;
                }
                let job = match receiver.recv_timeout(Duration::from_millis(5)) {
                    Ok(job) => job,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                let output_keyboard = keyboard::Keyboard::open();
                let result = if let Ok(ref keyboard) = output_keyboard {
                    let mut edit = Edit {
                        connection: effects.clone(),
                        clipboard: &mut clipboard,
                        keyboard,
                        shared: &worker_shared,
                        expected: worker_expected.clone(),
                        target: job.target,
                        generation: job.generation,
                        settings: &worker_settings,
                        keys: Vec::new(),
                        restore: None,
                        submitted: false,
                        leased: false,
                        field_context: worker_context.clone(),
                        field: job.field,
                        synthetic: policy::SyntheticKeys::default(),
                        connections: worker_connections.clone(),
                    };
                    let result = (|| {
                        let _guard = worker_shared.effects.lock().map_err(fail)?;
                        if job.expansion.rule.rich_html.is_some() {
                            return Err(ProductError::InputBlocked);
                        }
                        effects::wait_for_modifier_release(
                            || edit.check(),
                            || thread::sleep(Duration::from_millis(5)),
                        )?;
                        effects::perform(&mut edit, &job.expansion, &worker_settings)
                    })();
                    // Always try ownership-safe cleanup, including cancellation/partial output.
                    let cleanup = edit.finish();
                    if let Ok(mut status) = worker_shared.status.lock() {
                        status.submitted =
                            status.submitted.saturating_add(u32::from(edit.submitted));
                        status.clipboard_incomplete = cleanup.is_err();
                    }
                    result.and(cleanup)
                } else {
                    Err(ProductError::Unavailable)
                };
                if let Ok(mut status) = worker_shared.status.lock() {
                    status.last_error = result.err();
                    if result == Err(ProductError::Cancelled) {
                        status.cancelled = status.cancelled.saturating_add(1);
                    }
                }
                worker_shared.generation.fetch_add(1, Ordering::SeqCst);
                worker_shared.busy.store(false, Ordering::SeqCst);
            }
            if !worker_stop.load(Ordering::SeqCst)
                && worker_shared.active.swap(false, Ordering::SeqCst)
            {
                worker_shared.generation.fetch_add(1, Ordering::SeqCst);
                if let Ok(mut status) = worker_shared.status.lock() {
                    status.phase = crate::product::service::ProductPhase::Fault;
                    status.last_error = Some(ProductError::Unavailable);
                }
                interrupt(&worker_connections);
            }
        })
        .map_err(fail)?;
    let result = (|| {
        let mut stream = recording.record_enable_context(context).map_err(fail)?;
        recording.flush().map_err(fail)?;
        let first = stream
            .next()
            .ok_or(ProductError::Unavailable)?
            .map_err(fail)?;
        if first.category != 4 {
            return Err(ProductError::Unavailable);
        }
        ready.send(Ok(())).map_err(fail)?;
        let mut history = TypedHistory::new(rules, settings.boundary, settings.activation);
        let mut previous: Option<Target> = None;
        let mut generation = shared.generation.load(Ordering::SeqCst);
        let mut field_token = None;
        let mut generated: VecDeque<(bool, u8, u8)> = VecDeque::new();
        for reply in stream {
            if stopped.load(Ordering::SeqCst) || !shared.active.load(Ordering::SeqCst) {
                break;
            }
            let reply = reply.map_err(fail)?;
            if reply.category == 1 {
                // RECORD orders the FakeInput request before its generated device event.
                // Only events attributed to our exact effect connection exclude themselves.
                // Foreign delayed XTEST cannot be safely classified as hardware.
                for (event, key) in policy::fake_inputs(
                    &reply.data,
                    reply.client_swapped,
                    xtest_opcode,
                    effects_base,
                    reply.xid_base,
                )
                .map_err(|()| ProductError::InputBlocked)?
                {
                    if generated.len() >= 64 {
                        return Err(ProductError::InputBlocked);
                    }
                    generated.push_back((true, event, key));
                }
                continue;
            }
            if reply.category != 0 {
                continue;
            }
            if reply.client_swapped || reply.data.len() > 64 * 1024 || reply.data.len() % 32 != 0 {
                invalidate(&shared, &mut history);
                continue;
            }
            for raw in reply.data.as_chunks::<32>().0 {
                let event_type = raw[0] & 0x7f;
                let code = raw[1];
                if generated
                    .front()
                    .is_some_and(|(_, event, key)| (*event, *key) == (event_type, code))
                {
                    let (own, _, _) = generated.pop_front().ok_or(ProductError::InputBlocked)?;
                    let mut pending = expected.lock().map_err(fail)?;
                    if own && pending.front() == Some(&(event_type, code)) {
                        pending.pop_front();
                        history.reset();
                        continue;
                    }
                    // Other clients' synthetic input never joins remembered hardware text.
                    invalidate(&shared, &mut history);
                    continue;
                }
                if !matches!(event_type, KEY_PRESS_EVENT | KEY_RELEASE_EVENT) {
                    invalidate(&shared, &mut history);
                    previous = None;
                    continue;
                }
                if event_type == KEY_RELEASE_EVENT {
                    continue;
                }
                if shared.busy.load(Ordering::SeqCst) {
                    invalidate(&shared, &mut history);
                    continue;
                }
                let event = KeyPressEvent::try_parse(raw).map_err(fail)?.0;
                if raw[0] & 0x80 != 0 {
                    invalidate(&shared, &mut history);
                    continue;
                }
                if u16::from(event.state) & 0xfc != 0 {
                    invalidate(&shared, &mut history);
                    continue;
                }
                for _ in 0..64 {
                    let Some(event) = control.poll_for_event().map_err(fail)? else {
                        break;
                    };
                    let changed = match event {
                        Event::XkbMapNotify(_) | Event::XkbNewKeyboardNotify(_) => true,
                        Event::XkbStateNotify(e) => u16::from(e.changed) & 0xf0 != 0,
                        _ => false,
                    };
                    if changed {
                        keyboard = keyboard::Keyboard::open()?;
                        invalidate(&shared, &mut history);
                        previous = None;
                    }
                }
                let current = target(&control)?;
                let current_generation = shared.generation.load(Ordering::SeqCst);
                if previous.as_ref() != Some(&current) || current_generation != generation {
                    if previous
                        .as_ref()
                        .is_some_and(|last| last.map != current.map)
                    {
                        keyboard = keyboard::Keyboard::open()?;
                    }
                    history.reset();
                    generation = current_generation;
                    previous = Some(current.clone());
                }
                if current.pid == std::process::id() {
                    history.reset();
                    continue;
                }
                let field = match field_context.snapshot(current.pid, generation) {
                    Ok(field) => field,
                    Err(_) => {
                        invalidate(&shared, &mut history);
                        field_token = None;
                        continue;
                    }
                };
                if field_token != Some(field.token) {
                    invalidate(&shared, &mut history);
                    generation = shared.generation.load(Ordering::SeqCst);
                    field_token = Some(field.token);
                }
                let state = (u16::from(event.state) & 0xff) | (u16::from(current.group) << 13);
                let Some(symbol) = keyboard.symbol(code, state) else {
                    invalidate(&shared, &mut history);
                    continue;
                };
                let expansion = if symbol == 0xff08 {
                    history.backspace_delivered()
                } else if let Some(character) = policy::ascii_keysym(symbol) {
                    history.push_delivered(character)
                } else if (0xffe1..=0xffee).contains(&symbol) {
                    continue;
                } else {
                    invalidate(&shared, &mut history);
                    None
                };
                if let Some(expansion) = expansion {
                    let suffix = format!(
                        "{}{}",
                        expansion.rule.trigger,
                        expansion
                            .terminator
                            .map(|ch| ch.to_string())
                            .unwrap_or_default()
                    );
                    if !field.suffix.ends_with(&suffix) {
                        invalidate(&shared, &mut history);
                        continue;
                    }
                    if shared
                        .busy
                        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                        && jobs
                            .try_send(Job {
                                expansion,
                                target: current,
                                generation,
                                field,
                            })
                            .is_err()
                    {
                        shared.busy.store(false, Ordering::SeqCst);
                        invalidate(&shared, &mut history);
                    }
                }
            }
        }
        Ok(())
    })();
    stopped.store(true, Ordering::SeqCst);
    shared.generation.fetch_add(1, Ordering::SeqCst);
    drop(jobs);
    let _ = control.record_disable_context(context);
    let _ = control.record_free_context(context);
    let _ = control.flush();
    let _ = worker.join();
    if !field_context.shutdown() {
        return Err(ProductError::Busy);
    }
    result
}

struct Edit<'a> {
    connection: Arc<RustConnection>,
    clipboard: &'a mut clipboard::Clipboard,
    keyboard: &'a keyboard::Keyboard,
    shared: &'a Shared,
    expected: Expected,
    target: Target,
    generation: u64,
    settings: &'a ProductSettings,
    keys: Vec<(u8, bool)>,
    restore: Option<std::collections::BTreeMap<Atom, clipboard::Format>>,
    submitted: bool,
    leased: bool,
    field_context: context::Guard,
    field: context::Evidence,
    synthetic: policy::SyntheticKeys,
    connections: Connections,
}
impl Edit<'_> {
    fn key_for(&self, character: char) -> Option<(u8, bool)> {
        // Resolve the entire payload against the qualified active map before deleting.
        policy::key_for(
            character,
            self.connection.setup().min_keycode,
            self.connection.setup().max_keycode,
            |code, shift| {
                let symbol = self.keyboard.symbol(
                    code,
                    (u16::from(self.target.group) << 13) | u16::from(shift),
                )?;
                match symbol {
                    0x20..=0x7e | 0xa0..=0xff => char::from_u32(symbol as u32),
                    0x01000100..=0x0110ffff => char::from_u32((symbol & 0xffffff) as u32),
                    _ => None,
                }
            },
        )
    }
    fn symbol_key(&self, symbol: u64) -> Result<u8, ProductError> {
        (self.connection.setup().min_keycode..=self.connection.setup().max_keycode)
            .find(|&key| {
                self.keyboard
                    .symbol(key, u16::from(self.target.group) << 13)
                    == Some(symbol)
            })
            .ok_or(ProductError::InputBlocked)
    }
    fn post(&mut self, code: u8, down: bool) -> Result<(), ProductError> {
        let event = if down {
            KEY_PRESS_EVENT
        } else {
            KEY_RELEASE_EVENT
        };
        let mut expected = self.expected.lock().map_err(fail)?;
        if expected.len() >= 64 {
            return Err(ProductError::InputBlocked);
        }
        expected.push_back((event, code));
        drop(expected);
        self.synthetic.before_send(code, down);
        self.submitted = true;
        let result = self
            .connection
            .xtest_fake_input(event, code, 0, 0, 0, 0, 0)
            .map_err(fail)
            .and_then(|cookie| cookie.check().map_err(fail));
        self.synthetic.acknowledge(code, down, result.is_ok());
        if result.is_err() {
            self.release_pending();
        }
        result
    }
    fn release_pending(&mut self) {
        let held = self.synthetic.held();
        if held.is_empty() {
            return;
        }
        // A release attempt repairs possible native state; it never restores
        // confidence in an uncertain session or authorizes a second job.
        self.synthetic.mark_uncertain();
        for key in held {
            if let Ok(mut expected) = self.expected.lock()
                && expected.len() < 64
            {
                expected.push_back((KEY_RELEASE_EVENT, key));
            }
            let result = self
                .connection
                .xtest_fake_input(KEY_RELEASE_EVENT, key, 0, 0, 0, 0, 0)
                .map_err(fail)
                .and_then(|cookie| cookie.check().map_err(fail));
            self.synthetic.acknowledge(key, false, result.is_ok());
        }
    }
    fn press(&mut self, key: u8, shift: bool) -> Result<(), ProductError> {
        self.check()?;
        let modifier = if shift {
            Some(self.symbol_key(0xffe1)?)
        } else {
            None
        };
        if let Some(modifier) = modifier {
            self.post(modifier, true)?;
        }
        let result = self.post(key, true).and_then(|()| self.post(key, false));
        // Release only the modifier this operation synthesized, including failure paths.
        let release = modifier
            .map(|modifier| self.post(modifier, false))
            .unwrap_or(Ok(()));
        result.and(release)?;
        self.connection.flush().map_err(fail)?;
        thread::sleep(Duration::from_secs_f64(
            2.0 / f64::from(self.settings.typing_speed.events_per_second()),
        ));
        Ok(())
    }
    fn finish(&mut self) -> Result<(), ProductError> {
        self.release_pending();
        if let Some(snapshot) = self.restore.take() {
            self.clipboard.restore(snapshot)?;
        }
        Ok(())
    }
}
impl Drop for Edit<'_> {
    fn drop(&mut self) {
        self.release_pending();
        if self.synthetic.uncertain() {
            self.shared.active.store(false, Ordering::SeqCst);
            self.shared.generation.fetch_add(1, Ordering::SeqCst);
            if let Ok(mut status) = self.shared.status.lock() {
                status.phase = crate::product::service::ProductPhase::Fault;
                status.last_error = Some(ProductError::InputBlocked);
            }
            // Only after best-effort releases: terminate RECORD/control/worker
            // reads and prevent another admitted job from this uncertain session.
            interrupt(&self.connections);
        }
    }
}
impl EffectBackend for Edit<'_> {
    fn check(&mut self) -> Result<(), ProductError> {
        if !self.shared.active.load(Ordering::SeqCst)
            || self.shared.generation.load(Ordering::SeqCst) != self.generation
        {
            return Err(ProductError::Cancelled);
        }
        if target(&self.connection)? != self.target {
            return Err(ProductError::TargetChanged);
        }
        let field = self
            .field_context
            .snapshot(self.target.pid, self.generation)?;
        if field.token != self.field.token {
            return Err(ProductError::TargetChanged);
        }
        let state = self
            .connection
            .xkb_get_state(CORE)
            .map_err(fail)?
            .reply()
            .map_err(fail)?;
        // Lock modifiers affect Type representability as well; never force-release them.
        if u16::from(state.mods) != 0 {
            return Err(ProductError::ModifiersHeld);
        }
        if self.leased && !self.clipboard.current()? {
            return Err(ProductError::Clipboard);
        }
        Ok(())
    }
    fn prepare(&mut self, text: &str, mode: InsertionMode) -> Result<(), ProductError> {
        self.check()?;
        let field = self
            .field_context
            .snapshot(self.target.pid, self.generation)?;
        if field.caret != self.field.caret || field.suffix != self.field.suffix {
            return Err(ProductError::TargetChanged);
        }
        if text.len() > policy::MAX_BYTES {
            return Err(ProductError::InputBlocked);
        }
        self.symbol_key(0xff08)?;
        if mode == InsertionMode::Type {
            self.keys = policy::prepare_text(text, |ch| self.key_for(ch))
                .ok_or(ProductError::InputBlocked)?;
        } else {
            // Preflight shortcut and all clipboard data before the first Backspace.
            self.symbol_key(0xffe3)?;
            self.key_for('v')
                .filter(|(_, shift)| !shift)
                .ok_or(ProductError::InputBlocked)?;
            let token = if self.settings.restore_clipboard {
                let (snapshot, token) = self.clipboard.snapshot()?;
                self.restore = Some(snapshot);
                token
            } else {
                self.clipboard.acquisition_token()?
            };
            self.check()?;
            self.clipboard.publish(text, token)?;
            self.leased = true;
        }
        Ok(())
    }
    fn erase(&mut self, characters: usize) -> Result<(), ProductError> {
        let backspace = self.symbol_key(0xff08)?;
        for _ in 0..characters {
            self.press(backspace, false)?;
        }
        Ok(())
    }
    fn content(&mut self, _text: &str, mode: InsertionMode) -> Result<(), ProductError> {
        if mode == InsertionMode::Type {
            for (key, shift) in self.keys.clone() {
                self.press(key, shift)?;
            }
        } else {
            self.check()?;
            let control = self.symbol_key(0xffe3)?;
            let (key, _) = self.key_for('v').ok_or(ProductError::InputBlocked)?;
            self.post(control, true)?;
            let result = self.post(key, true).and_then(|()| self.post(key, false));
            let release = self.post(control, false);
            result.and(release)?;
            let deadline = Instant::now() + Duration::from_millis(200);
            while Instant::now() < deadline {
                self.clipboard.pump()?;
                self.check()?;
                thread::sleep(Duration::from_millis(2));
            }
            // A bounded settling window is submission evidence, never editor acknowledgement.
        }
        Ok(())
    }
    fn terminator(&mut self, character: char) -> Result<(), ProductError> {
        let (key, shift) = self.key_for(character).ok_or(ProductError::InputBlocked)?;
        self.press(key, shift)
    }
}
