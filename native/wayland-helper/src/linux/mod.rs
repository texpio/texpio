//! Linux-only native bindings. Nothing here runs in unit tests.
mod attestation;
mod broker;
mod bus;
mod devices;
mod field;
use crate::*;
pub use broker::serve;
pub use field::Tracker as FieldTracker;
use std::{io, os::unix::net::UnixStream, time::Duration};

pub struct Client {
    stream: UnixStream,
    pub generation: u64,
    pub clock_ms: u64,
}
impl Client {
    pub fn probe() -> Result<(), Refusal> {
        Self::handshake(true).map(|_| ())
    }
    pub fn connect() -> Result<Self, Refusal> {
        Self::handshake(false)
    }
    fn handshake(status: bool) -> Result<Self, Refusal> {
        let mut stream = UnixStream::connect(SOCKET).map_err(|_| Refusal::Session)?;
        stream
            .set_read_timeout(Some(Duration::from_millis(1500)))
            .map_err(|_| Refusal::Protocol)?;
        stream
            .set_write_timeout(Some(Duration::from_millis(500)))
            .map_err(|_| Refusal::Protocol)?;
        let credentials = broker::credentials(&stream).map_err(|_| Refusal::Session)?;
        if credentials.uid != 0 {
            return Err(Refusal::Session);
        }
        write_frame(
            &mut stream,
            &if status {
                Request::Status { version: VERSION }
            } else {
                Request::Hello { version: VERSION }
            },
        )
        .map_err(|_| Refusal::Protocol)?;
        match read_frame(&mut stream).map_err(|_| Refusal::Protocol)? {
            Reply::Ready {
                version: VERSION,
                generation,
                monotonic_ms,
            } if generation != 0 => Ok(Self {
                stream,
                generation,
                clock_ms: monotonic_ms,
            }),
            Reply::Refused { reason } => Err(reason),
            _ => Err(Refusal::Protocol),
        }
    }
    pub fn exchange(&mut self, request: Request) -> Result<Reply, Refusal> {
        write_frame(&mut self.stream, &request).map_err(|_| Refusal::Protocol)?;
        read_frame(&mut self.stream).map_err(|_| Refusal::Protocol)
    }
    pub fn close(&self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
    pub fn interrupter(&self) -> io::Result<UnixStream> {
        self.stream.try_clone()
    }
}
pub fn now_ms() -> u64 {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::zeroed();
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, value.as_mut_ptr()) } != 0 {
        return 0;
    }
    let value = unsafe { value.assume_init() };
    value.tv_sec as u64 * 1000 + value.tv_nsec as u64 / 1_000_000
}

/// Read-only logind guard for the caller, usable by the X11 adapter too.
/// A missing/ambiguous session or unavailable lock hint is a refusal.
pub fn session_unlocked() -> bool {
    broker::current_session_unlocked()
}
