//! Bounded native-only protocol. Never export these messages to the webview.
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
pub mod attestation;
pub const VERSION: u32 = 1;
pub const MAX_FRAME: usize = 64 * 1024;
pub const MAX_TEXT: usize = 16 * 1024;
pub const HEARTBEAT_MS: u64 = 2000;
pub const SOCKET: &str = "/run/texpio/wayland.sock";

pub fn traverse_children(depth: usize, object_pid: u32, target_pid: u32) -> bool {
    depth == 0 || object_pid == target_pid
}
/// Physical keys already released can still be queued. A zero key bitmap does
/// not authorize ignoring those actions. Only the completing key's release is
/// admissible during settling, and no physical action is admissible afterward.
pub struct ObservationGate {
    pending_release: Option<u16>,
}
impl ObservationGate {
    pub fn trigger_release(code: u16) -> Self {
        Self {
            pending_release: Some(code),
        }
    }
    pub fn quiet() -> Self {
        Self {
            pending_release: None,
        }
    }
    pub fn accept(&mut self, kind: u16, code: u16, value: i32) -> bool {
        if (kind, code, value) == (0, 0, 0) {
            return true;
        }
        if kind == 1 && value == 0 && self.pending_release == Some(code) {
            self.pending_release = None;
            return true;
        }
        false
    }
}

/// A transport error permanently poisons this device lifetime. Key-up is always
/// attempted for keys whose down event may have reached the kernel.
pub trait KeyOutput {
    fn event(&mut self, code: u16, down: bool) -> io::Result<()>;
    fn sync(&mut self) -> io::Result<()>;
}
pub struct SyntheticKeys<S: KeyOutput> {
    sink: S,
    held: std::collections::BTreeSet<u16>,
    poisoned: bool,
}
impl<S: KeyOutput> SyntheticKeys<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink,
            held: Default::default(),
            poisoned: false,
        }
    }
    pub fn poisoned(&self) -> bool {
        self.poisoned
    }
    pub fn key(&mut self, code: u16, down: bool) -> io::Result<()> {
        if self.poisoned {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        // Keep a key recorded until both its release and SYN_REPORT succeed.
        self.held.insert(code);
        if let Err(error) = self.sink.event(code, down).and_then(|_| self.sink.sync()) {
            self.poisoned = true;
            self.release();
            return Err(error);
        }
        if !down {
            self.held.remove(&code);
        }
        Ok(())
    }
    pub fn tap(&mut self, code: u16) -> io::Result<()> {
        self.key(code, true)?;
        self.key(code, false)
    }
    pub fn cancel(&mut self) {
        self.poisoned = true;
        self.release();
    }
    fn release(&mut self) {
        for code in self.held.iter().copied().collect::<Vec<_>>() {
            let up = self.sink.event(code, false);
            let sync = self.sink.sync();
            if up.is_ok() && sync.is_ok() {
                self.held.remove(&code);
            }
        }
    }
}
impl<S: KeyOutput> Drop for SyntheticKeys<S> {
    fn drop(&mut self) {
        self.release();
    }
}

pub fn approved_device(sys_path: &str, udev: &str) -> bool {
    sys_path.starts_with("/sys/devices/")
        && !sys_path.contains("/devices/virtual/")
        && !udev
            .lines()
            .any(|l| l.starts_with("E:ID_SEAT=") && l != "E:ID_SEAT=seat0")
        && udev.lines().any(|l| {
            matches!(
                l,
                "E:ID_INPUT_KEYBOARD=1" | "E:ID_INPUT_MOUSE=1" | "E:ID_INPUT_TOUCHPAD=1"
            )
        })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub uid: u32,
    pub pid: i32,
    pub id: String,
    pub seat: String,
    pub active: bool,
    pub local: bool,
    pub user: bool,
    pub wayland: bool,
    pub unlocked: bool,
}
pub fn authorize(expected: &Session, actual: &Session) -> bool {
    expected.uid != 0
        && expected.pid > 0
        && expected == actual
        && actual.active
        && actual.local
        && actual.user
        && actual.wayland
        && actual.unlocked
        && !actual.id.is_empty()
        && actual.seat == "seat0"
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Refusal {
    Protocol,
    Session,
    Locked,
    BridgeMissing,
    BridgeAuthentication,
    Context,
    CompositionUnknown,
    Field,
    Generation,
    Deadline,
    Modifiers,
    Clipboard,
    Unsupported,
    Input,
    Uncertain,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Hello {
        version: u32,
    },
    Status {
        version: u32,
    },
    Poll {
        generation: u64,
    },
    Cancel {
        generation: u64,
    },
    Replace {
        generation: u64,
        deadline_ms: u64,
        context_epoch: u64,
        field: u64,
        pid: u32,
        suffix: String,
        text: String,
        paste: bool,
        restore: bool,
        pace_ms: u32,
    },
}
impl Request {
    pub fn valid_hello(&self) -> bool {
        matches!(self, Self::Hello { version: VERSION })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "camelCase", deny_unknown_fields)]
pub enum Reply {
    Ready {
        version: u32,
        generation: u64,
        monotonic_ms: u64,
    },
    Idle,
    Input {
        code: u16,
        character: Option<char>,
        context_epoch: u64,
        field: u64,
        pid: u32,
    },
    Reset,
    Submitted,
    Refused {
        reason: Refusal,
    },
}
pub fn read_frame<T: for<'a> Deserialize<'a>>(reader: &mut impl Read) -> io::Result<T> {
    let mut prefix = [0; 4];
    reader.read_exact(&mut prefix)?;
    let len = u32::from_be_bytes(prefix) as usize;
    if len == 0 || len > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame size"));
    }
    let mut data = vec![0; len];
    reader.read_exact(&mut data)?;
    serde_json::from_slice(&data)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "frame schema"))
}
pub fn write_frame(writer: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let data = serde_json::to_vec(value).map_err(io::Error::other)?;
    if data.is_empty() || data.len() > MAX_FRAME {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame size"));
    }
    writer.write_all(&(data.len() as u32).to_be_bytes())?;
    writer.write_all(&data)?;
    writer.flush()
}
#[derive(Debug)]
pub struct Lease {
    generation: u64,
    last_heartbeat: u64,
    revoked: bool,
}
impl Lease {
    pub fn new(generation: u64, now: u64) -> Self {
        Self {
            generation,
            last_heartbeat: now,
            revoked: generation == 0,
        }
    }
    pub fn live(&self, generation: u64, now: u64) -> bool {
        !self.revoked
            && generation == self.generation
            && now >= self.last_heartbeat
            && now - self.last_heartbeat <= HEARTBEAT_MS
    }
    pub fn heartbeat(&mut self, generation: u64, now: u64) -> bool {
        if !self.live(generation, now) {
            self.revoke();
            return false;
        }
        self.last_heartbeat = now;
        true
    }
    pub fn accept(&mut self, generation: u64, now: u64, deadline: u64) -> bool {
        self.live(generation, now) && deadline >= now && deadline - now <= 1000
    }
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Composition {
    Idle,
    Active,
    Unknown,
}
/// Evidence is obtained locally via AT-SPI; the client cannot supply it.
#[derive(Clone, Debug)]
pub struct FieldEvidence {
    pub token: u64,
    pub context_epoch: u64,
    pub pid: u32,
    pub caret: i32,
    pub suffix: String,
    pub focused: bool,
    pub editable: bool,
    pub protected: bool,
    pub composition: Composition,
    pub selections: i32,
}
impl FieldEvidence {
    pub fn safe(&self) -> bool {
        self.token != 0
            && self.pid != 0
            && self.caret >= 0
            && self.focused
            && self.editable
            && !self.protected
            && self.composition == Composition::Idle
            && self.selections == 0
    }
    pub fn verify(&self, token: u64, epoch: u64, pid: u32, suffix: &str) -> Result<(), Refusal> {
        if self.composition == Composition::Unknown {
            return Err(Refusal::CompositionUnknown);
        }
        if !self.safe()
            || token != self.token
            || epoch != self.context_epoch
            || pid != self.pid
            || suffix.is_empty()
            || suffix.len() > 512
            || !suffix.is_ascii()
            || suffix.chars().any(char::is_control)
            || self.caret < suffix.len() as i32
            || !self.suffix.ends_with(suffix)
        {
            return Err(Refusal::Field);
        }
        Ok(())
    }
    pub fn erasure_observed(&self, before: &Self, count: usize) -> bool {
        self.safe()
            && before.safe()
            && self.token == before.token
            && self.context_epoch == before.context_epoch
            && self.pid == before.pid
            && self.caret == before.caret - count as i32
            && before.suffix.len() >= count
            && before
                .suffix
                .get(..before.suffix.len() - count)
                .is_some_and(|prefix| self.suffix.ends_with(prefix))
    }
}
#[cfg(target_os = "linux")]
pub mod linux;
