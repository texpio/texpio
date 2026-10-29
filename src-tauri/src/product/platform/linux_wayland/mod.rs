//! Native client for the separately installed session-scoped Wayland helper.
use crate::product::{
    effects::ProductError,
    matcher::{ProductRule, TypedHistory},
    service::{ProductPhase, Shared},
    settings::{InsertionMode, ProductSettings},
};
use std::{
    os::unix::net::UnixStream,
    sync::{Arc, atomic::Ordering},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use texpio_wayland_helper::{
    MAX_TEXT, Refusal, Reply, Request,
    linux::{Client, now_ms},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Readiness {
    HelperMissing,
    BridgeMissing,
    BridgeAuthenticationUnavailable,
    SessionUnavailable,
    FieldEvidenceUnavailable,
    Ready,
}
pub(crate) fn readiness() -> Readiness {
    if !std::path::Path::new(texpio_wayland_helper::SOCKET).exists() {
        return Readiness::HelperMissing;
    }
    match Client::probe() {
        Ok(()) => Readiness::Ready,
        Err(Refusal::BridgeMissing) => Readiness::BridgeMissing,
        Err(Refusal::BridgeAuthentication) => Readiness::BridgeAuthenticationUnavailable,
        Err(Refusal::CompositionUnknown | Refusal::Field | Refusal::Context) => {
            Readiness::FieldEvidenceUnavailable
        }
        Err(_) => Readiness::SessionUnavailable,
    }
}
pub(crate) struct Runtime {
    worker: Option<JoinHandle<()>>,
    interrupt: UnixStream,
}
impl Runtime {
    pub(crate) fn launch(
        shared: Arc<Shared>,
        rules: Vec<ProductRule>,
        settings: ProductSettings,
    ) -> Result<Self, ProductError> {
        if settings.allow_software_input {
            return Err(ProductError::InvalidSettings);
        }
        // No observer exists until the helper has proved session, bridge and field prerequisites.
        let client = Client::connect().map_err(error)?;
        let interrupt = client
            .interrupter()
            .map_err(|_| ProductError::Unavailable)?;
        let generation = shared.generation.load(Ordering::SeqCst);
        let worker = thread::Builder::new()
            .name("texpio-wayland".into())
            .spawn(move || run(client, shared, generation, rules, settings))
            .map_err(|_| ProductError::Unavailable)?;
        Ok(Self {
            worker: Some(worker),
            interrupt,
        })
    }
    pub(crate) fn shutdown(&mut self) -> bool {
        let _ = self.interrupt.shutdown(std::net::Shutdown::Both);
        let until = Instant::now() + Duration::from_secs(2);
        while self
            .worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
            && Instant::now() < until
        {
            thread::sleep(Duration::from_millis(5));
        }
        if self
            .worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
        {
            return false;
        }
        self.worker
            .take()
            .is_none_or(|worker| worker.join().is_ok())
    }
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
fn error(reason: Refusal) -> ProductError {
    match reason {
        Refusal::Generation | Refusal::Deadline => ProductError::Cancelled,
        Refusal::Modifiers => ProductError::ModifiersHeld,
        Refusal::Clipboard => ProductError::Clipboard,
        Refusal::Context | Refusal::Field => ProductError::TargetChanged,
        Refusal::Input | Refusal::Uncertain => ProductError::InputBlocked,
        Refusal::Unsupported => ProductError::InvalidSettings,
        _ => ProductError::Unavailable,
    }
}
fn cancel_code(value: &str) -> u16 {
    match value {
        "Escape" => 1,
        "Pause" => 119,
        "ScrollLock" => 70,
        _ => value
            .strip_prefix('F')
            .and_then(|v| v.parse::<u16>().ok())
            .map(|n| if n <= 10 { 58 + n } else { 76 + n })
            .unwrap_or(1),
    }
}
fn run(
    mut client: Client,
    shared: Arc<Shared>,
    generation: u64,
    rules: Vec<ProductRule>,
    settings: ProductSettings,
) {
    let rules = rules
        .into_iter()
        .filter(|r| {
            r.trigger.is_ascii()
                && !r.trigger.chars().any(char::is_control)
                && r.rich_html.is_none()
                && r.body.len() <= MAX_TEXT
        })
        .collect();
    let mut history = TypedHistory::new(rules, settings.boundary, settings.activation);
    let mut target = None;
    let active = || {
        shared.active.load(Ordering::SeqCst)
            && shared.generation.load(Ordering::SeqCst) == generation
    };
    let mut fault = None;
    while active() {
        let event = match client.exchange(Request::Poll {
            generation: client.generation,
        }) {
            Ok(e) => e,
            Err(e) => {
                fault = Some(error(e));
                break;
            }
        };
        let candidate = match event {
            Reply::Idle => continue,
            Reply::Reset => {
                history.reset();
                target = None;
                continue;
            }
            Reply::Refused { reason } => {
                fault = Some(error(reason));
                break;
            }
            Reply::Input {
                code,
                character,
                context_epoch,
                field,
                pid,
            } => {
                let stamp = (context_epoch, field, pid);
                if target != Some(stamp) {
                    history.reset();
                    target = Some(stamp)
                }
                if code == cancel_code(&settings.cancel_key) {
                    history.reset();
                    continue;
                }
                if code == 14 {
                    history.backspace_delivered()
                } else if let Some(ch) = character.filter(|c| c.is_ascii() && !c.is_control()) {
                    history.push_delivered(ch)
                } else {
                    history.reset();
                    None
                }
            }
            _ => {
                fault = Some(ProductError::Unavailable);
                break;
            }
        };
        let Some(expansion) = candidate else { continue };
        let Some((epoch, field, pid)) = target else {
            continue;
        };
        if shared
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            history.reset();
            continue;
        }
        let result = (|| {
            let _effects = shared.effects.lock().map_err(|_| ProductError::Busy)?;
            if !active() {
                return Err(ProductError::Cancelled);
            }
            if let Some(resolve) = shared
                .files
                .lock()
                .map_err(|_| ProductError::Unavailable)?
                .as_ref()
            {
                let files = resolve(&expansion.rule.id)?;
                if !files.files.is_empty() {
                    return Err(ProductError::Unavailable);
                }
            }
            let mut suffix = expansion.rule.trigger.clone();
            if let Some(end) = expansion.terminator {
                suffix.push(end)
            }
            if suffix.len() != expansion.erase_count {
                return Err(ProductError::InputBlocked);
            }
            let mut text = expansion.rule.body.to_string();
            if settings.preserve_terminator
                && let Some(end) = expansion.terminator
            {
                text.push(end)
            }
            let reply = client
                .exchange(Request::Replace {
                    generation: client.generation,
                    deadline_ms: now_ms() + 1000,
                    context_epoch: epoch,
                    field,
                    pid,
                    suffix,
                    text,
                    paste: settings.insertion == InsertionMode::Paste,
                    restore: settings.restore_clipboard,
                    pace_ms: (1000 / settings.typing_speed.events_per_second()).clamp(2, 20),
                })
                .map_err(error)?;
            match reply {
                Reply::Submitted => Ok(()),
                Reply::Refused { reason } => Err(error(reason)),
                _ => Err(ProductError::InputBlocked),
            }
        })();
        shared.busy.store(false, Ordering::SeqCst);
        history.reset();
        target = None;
        let mut status = shared.status.lock().unwrap_or_else(|e| e.into_inner());
        match result {
            Ok(()) => {
                status.submitted = status.submitted.saturating_add(1);
                status.last_error = None;
                drop(status);
                if let Some(record) = shared
                    .used
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .as_ref()
                {
                    record(&expansion.rule.id)
                }
            }
            Err(error) => {
                status.cancelled = status.cancelled.saturating_add(1);
                status.last_error = Some(error)
            }
        }
    }
    client.close();
    if let Some(error) = fault
        && active()
    {
        shared.active.store(false, Ordering::SeqCst);
        let mut status = shared.status.lock().unwrap_or_else(|e| e.into_inner());
        status.phase = ProductPhase::Fault;
        status.last_error = Some(error);
    }
}
