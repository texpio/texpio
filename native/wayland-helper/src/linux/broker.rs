use super::{
    attestation::PublishedAttestation,
    bus::Bus,
    devices::{Devices, Keymap, device_paths},
    field::Tracker,
    now_ms,
};
use crate::*;
use std::{
    ffi::{CStr, c_char, c_int},
    io::{self, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::net::{UnixListener, UnixStream},
    },
    ptr,
    time::Duration,
};
#[link(name = "systemd")]
unsafe extern "C" {
    fn sd_pid_get_session(pid: i32, s: *mut *mut c_char) -> c_int;
    fn sd_session_get_uid(s: *const c_char, u: *mut u32) -> c_int;
    fn sd_session_get_seat(s: *const c_char, v: *mut *mut c_char) -> c_int;
    fn sd_session_get_type(s: *const c_char, v: *mut *mut c_char) -> c_int;
    fn sd_session_get_class(s: *const c_char, v: *mut *mut c_char) -> c_int;
    fn sd_session_is_active(s: *const c_char) -> c_int;
    fn sd_session_is_remote(s: *const c_char) -> c_int;
}
pub(super) fn credentials(stream: &UnixStream) -> io::Result<libc::ucred> {
    let mut value = std::mem::MaybeUninit::<libc::ucred>::zeroed();
    let mut size = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    if unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            value.as_mut_ptr().cast(),
            &mut size,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { value.assume_init() })
}
fn session(peer: &libc::ucred, bus: &Bus) -> Result<Session, Refusal> {
    unsafe {
        let mut id = ptr::null_mut();
        if sd_pid_get_session(peer.pid, &mut id) < 0 || id.is_null() {
            return Err(Refusal::Session);
        }
        let get =
            |f: unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int| -> Option<String> {
                let mut p = ptr::null_mut();
                if f(id, &mut p) < 0 || p.is_null() {
                    return None;
                }
                let value = CStr::from_ptr(p).to_string_lossy().into_owned();
                libc::free(p.cast());
                Some(value)
            };
        let mut uid = 0;
        let id_string = CStr::from_ptr(id).to_string_lossy().into_owned();
        let result = Session {
            uid: if sd_session_get_uid(id, &mut uid) >= 0 {
                uid
            } else {
                0
            },
            pid: peer.pid,
            id: id_string.clone(),
            seat: get(sd_session_get_seat).unwrap_or_default(),
            active: sd_session_is_active(id) > 0,
            local: sd_session_is_remote(id) == 0,
            user: get(sd_session_get_class).as_deref() == Some("user"),
            wayland: get(sd_session_get_type).as_deref() == Some("wayland"),
            unlocked: bus.unlocked(&id_string),
        };
        libc::free(id.cast());
        if result.uid != peer.uid || !authorize(&result, &result) {
            return Err(Refusal::Session);
        }
        Ok(result)
    }
}
/// Reject SCM_RIGHTS rather than silently accepting descriptor-bearing frames.
struct NoRights<'a> {
    stream: &'a UnixStream,
    deadline: std::time::Instant,
}
impl Read for NoRights<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let remaining = self
            .deadline
            .checked_duration_since(std::time::Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or(io::ErrorKind::TimedOut)?;
        self.stream.set_read_timeout(Some(remaining))?;
        unsafe {
            let mut io = libc::iovec {
                iov_base: bytes.as_mut_ptr().cast(),
                iov_len: bytes.len(),
            };
            let mut control = [0usize; 32];
            let mut message: libc::msghdr = std::mem::zeroed();
            message.msg_iov = &mut io;
            message.msg_iovlen = 1;
            message.msg_control = control.as_mut_ptr().cast();
            message.msg_controllen = std::mem::size_of_val(&control);
            let count = libc::recvmsg(
                self.stream.as_raw_fd(),
                &mut message,
                libc::MSG_CMSG_CLOEXEC,
            );
            if count < 0 {
                return Err(io::Error::last_os_error());
            }
            let mut header = libc::CMSG_FIRSTHDR(&message);
            let unexpected = !header.is_null() || message.msg_flags & libc::MSG_CTRUNC != 0;
            while !header.is_null() {
                if (*header).cmsg_level == libc::SOL_SOCKET
                    && (*header).cmsg_type == libc::SCM_RIGHTS
                {
                    let length = (*header)
                        .cmsg_len
                        .saturating_sub(libc::CMSG_LEN(0) as usize)
                        / std::mem::size_of::<c_int>();
                    let data = libc::CMSG_DATA(header).cast::<c_int>();
                    for i in 0..length {
                        libc::close(*data.add(i));
                    }
                }
                header = libc::CMSG_NXTHDR(&message, header);
            }
            if unexpected {
                return Err(io::ErrorKind::InvalidData.into());
            }
            Ok(count as usize)
        }
    }
}
fn request(stream: &UnixStream) -> io::Result<Request> {
    read_frame(&mut NoRights {
        stream,
        deadline: std::time::Instant::now() + Duration::from_millis(750),
    })
}
fn drop_privilege(peer: &libc::ucred) -> io::Result<()> {
    unsafe {
        for capability in 0..=63 {
            libc::prctl(libc::PR_CAPBSET_DROP, capability, 0, 0, 0);
        }
        if libc::setgroups(0, ptr::null()) != 0
            || libc::setresgid(peer.gid, peer.gid, peer.gid) != 0
            || libc::setresuid(peer.uid, peer.uid, peer.uid) != 0
            || libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0
            || libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) != 0
        {
            return Err(io::Error::last_os_error());
        }
        if libc::geteuid() != peer.uid
            || libc::getegid() != peer.gid
            || libc::getgroups(0, ptr::null_mut()) != 0
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
            return Err(io::Error::last_os_error());
        }
        std::env::set_var("XDG_RUNTIME_DIR", format!("/run/user/{}", peer.uid));
        std::env::set_var(
            "DBUS_SESSION_BUS_ADDRESS",
            format!("unix:path=/run/user/{}/bus", peer.uid),
        );
    }
    Ok(())
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Context {
    version: u32,
    epoch: u64,
    pid: u32,
    unlocked: bool,
    overview: bool,
    source_type: String,
    layout: String,
    options: String,
}
fn context(bus: &Bus) -> Result<Context, Refusal> {
    let value = bus.bridge(serde_json::json!({"version":VERSION,"op":"context"}))?;
    let result: Context = serde_json::from_value(value).map_err(|_| Refusal::Context)?;
    if result.version != VERSION
        || result.epoch == 0
        || result.pid == 0
        || !result.unlocked
        || result.overview
        || result.source_type != "xkb"
    {
        return Err(Refusal::Context);
    }
    Ok(result)
}
fn kill_and_reap(child: i32) {
    unsafe {
        libc::kill(child, libc::SIGKILL);
    }
    loop {
        let result = unsafe { libc::waitpid(child, ptr::null_mut(), 0) };
        if result >= 0 || io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
            break;
        }
    }
}
/// Socket activated root broker. Only the child ever reads devices; the parent
/// independently checks the originating logind session and peer lifetime.
pub fn serve() -> io::Result<()> {
    if unsafe { libc::geteuid() } != 0
        || std::env::var("LISTEN_FDS").ok().as_deref() != Some("1")
        || std::env::var("LISTEN_PID").ok() != Some(std::process::id().to_string())
    {
        return Err(io::ErrorKind::PermissionDenied.into());
    }
    let listener = unsafe { UnixListener::from_raw_fd(3) };
    let bus = Bus::open(true).map_err(|_| io::ErrorKind::PermissionDenied)?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        stream.set_read_timeout(Some(Duration::from_millis(750)))?;
        stream.set_write_timeout(Some(Duration::from_millis(500)))?;
        let Ok(peer) = credentials(&stream) else {
            continue;
        };
        let Ok(expected) = session(&peer, &bus) else {
            let _ = write_frame(
                &mut stream,
                &Reply::Refused {
                    reason: Refusal::Session,
                },
            );
            continue;
        };
        let Ok(first) = request(&stream) else {
            continue;
        };
        let status = matches!(first, Request::Status { version: VERSION });
        if !status && !first.valid_hello() {
            let _ = write_frame(
                &mut stream,
                &Reply::Refused {
                    reason: Refusal::Protocol,
                },
            );
            continue;
        }
        let devices = if status {
            None
        } else {
            match Devices::open() {
                Ok(d) => Some(d),
                Err(_) => {
                    let _ = write_frame(
                        &mut stream,
                        &Reply::Refused {
                            reason: Refusal::Input,
                        },
                    );
                    continue;
                }
            }
        };
        let (mut publication, mut awaited) = UnixStream::pair()?;
        publication.set_write_timeout(Some(Duration::from_millis(500)))?;
        awaited.set_read_timeout(Some(Duration::from_millis(500)))?;
        let pidfd = unsafe { libc::syscall(libc::SYS_pidfd_open, peer.pid, 0) } as i32;
        if pidfd < 0 {
            continue;
        }
        let broker_pid = unsafe { libc::getpid() };
        let child = unsafe { libc::fork() };
        if child == 0 {
            unsafe {
                libc::close(listener.as_raw_fd());
                libc::close(pidfd);
            }
            drop(bus);
            drop(publication);
            let mut published = [0];
            let acknowledged = awaited.read_exact(&mut published).is_ok() && published == [1];
            drop(awaited);
            let code = if acknowledged
                && drop_privilege(&peer).is_ok()
                && unsafe { libc::getppid() } == broker_pid
            {
                match worker(&mut stream, devices, expected) {
                    Ok(()) => 0,
                    Err(reason) => {
                        let _ = write_frame(&mut stream, &Reply::Refused { reason });
                        1
                    }
                }
            } else {
                1
            };
            unsafe { libc::_exit(code) }
        }
        drop(devices);
        drop(awaited);
        if child < 0 {
            unsafe {
                libc::close(pidfd);
            }
            continue;
        }
        let attestation = PublishedAttestation::publish(child, expected.uid);
        if attestation.is_err() || publication.write_all(&[1]).is_err() {
            kill_and_reap(child);
            unsafe {
                libc::close(pidfd);
            }
            let _ = write_frame(
                &mut stream,
                &Reply::Refused {
                    reason: Refusal::BridgeAuthentication,
                },
            );
            continue;
        }
        // Holding this guard until reap removes the record on ordinary exit,
        // disconnect, lock, inactive session, or revocation. A crash-stale record
        // cannot authenticate a reused PID because kernel starttime must match.
        let _attestation = attestation?;
        drop(publication);
        loop {
            let mut status_code = 0;
            let waited = unsafe { libc::waitpid(child, &mut status_code, libc::WNOHANG) };
            if waited == child {
                break;
            }
            if waited < 0 {
                if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                kill_and_reap(child);
                break;
            }
            let mut poll = libc::pollfd {
                fd: pidfd,
                events: libc::POLLIN,
                revents: 0,
            };
            let dead = unsafe { libc::poll(&mut poll, 1, 0) } > 0;
            let mut connection = libc::pollfd {
                fd: stream.as_raw_fd(),
                events: libc::POLLRDHUP | libc::POLLHUP | libc::POLLERR,
                revents: 0,
            };
            let disconnected =
                unsafe { libc::poll(&mut connection, 1, 0) } < 0 || connection.revents != 0;
            if dead
                || disconnected
                || !session(&peer, &bus).is_ok_and(|actual| authorize(&expected, &actual))
            {
                kill_and_reap(child);
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        unsafe {
            libc::close(pidfd);
        }
    }
    Ok(())
}
fn worker(
    stream: &mut UnixStream,
    mut devices: Option<Devices>,
    expected: Session,
) -> Result<(), Refusal> {
    let bus = Bus::open(false)?;
    let ctx = context(&bus)?;
    let mut tracker = Tracker::new()?;
    let evidence = tracker.snapshot(ctx.pid, ctx.epoch)?;
    if evidence.composition == Composition::Unknown {
        return Err(Refusal::CompositionUnknown);
    }
    if !evidence.safe() {
        return Err(Refusal::Field);
    }
    let generation = now_ms()
        .checked_mul(65536)
        .and_then(|n| n.checked_add(std::process::id() as u64 % 65536))
        .ok_or(Refusal::Generation)?;
    write_frame(
        stream,
        &Reply::Ready {
            version: VERSION,
            generation,
            monotonic_ms: now_ms(),
        },
    )
    .map_err(|_| Refusal::Protocol)?;
    let Some(ref mut devices) = devices else {
        return Ok(());
    };
    let mut keymap = Keymap::new(&ctx.layout, &ctx.options)?;
    let mut lease = Lease::new(generation, now_ms());
    let system = Bus::open(true)?;
    let mut epoch = ctx.epoch;
    let mut last_devices = now_ms();
    let mut completing_key = None;
    loop {
        if !system.unlocked(&expected.id) {
            return Err(Refusal::Locked);
        }
        if now_ms().saturating_sub(last_devices) > 500 {
            if device_paths().map_err(|_| Refusal::Input)? != devices.paths {
                return Err(Refusal::Input);
            }
            last_devices = now_ms();
        }
        let req = request(stream).map_err(|_| Refusal::Protocol)?;
        let reply = match req {
            Request::Poll { generation: g } => {
                if !lease.heartbeat(g, now_ms()) {
                    return Err(Refusal::Generation);
                }
                let current = context(&bus)?;
                if current.epoch != epoch {
                    completing_key = None;
                    epoch = current.epoch;
                    keymap = Keymap::new(&current.layout, &current.options)?;
                    devices.require_released().map_err(|_| Refusal::Modifiers)?;
                    Reply::Reset
                } else {
                    let deadline = now_ms() + 40;
                    let mut result = Reply::Idle;
                    while now_ms() < deadline {
                        match devices.next().map_err(|_| Refusal::Input)? {
                            Some(event) => {
                                if event.kind == 0 && event.code == 3 {
                                    return Err(Refusal::Input);
                                } // SYN_DROPPED: close; fresh handshake/resync required.
                                if event.kind == 1 && event.code < 256 {
                                    let ch = keymap.update(event);
                                    if event.value != 0 {
                                        let field = tracker.snapshot(current.pid, current.epoch)?;
                                        if !field.safe() {
                                            return Err(Refusal::Field);
                                        }
                                        result = Reply::Input {
                                            code: event.code,
                                            character: ch,
                                            context_epoch: current.epoch,
                                            field: field.token,
                                            pid: current.pid,
                                        };
                                        completing_key = Some(event.code);
                                        break;
                                    }
                                } else if event.kind != 0 {
                                    completing_key = None;
                                    result = Reply::Reset;
                                    break;
                                }
                            }
                            None => std::thread::sleep(Duration::from_millis(2)),
                        }
                    }
                    result
                }
            }
            Request::Cancel { generation: g } if g == generation => {
                lease.revoke();
                return Ok(());
            }
            Request::Replace {
                generation: g,
                deadline_ms,
                context_epoch,
                field,
                pid,
                suffix,
                text,
                paste,
                restore,
                pace_ms,
            } => {
                if !lease.accept(g, now_ms(), deadline_ms) {
                    return Err(Refusal::Deadline);
                }
                let result = replace(
                    stream,
                    completing_key.take().ok_or(Refusal::Input)?,
                    &bus,
                    &system,
                    &expected,
                    &mut tracker,
                    devices,
                    &mut keymap,
                    deadline_ms,
                    context_epoch,
                    field,
                    pid,
                    &suffix,
                    &text,
                    paste,
                    restore,
                    pace_ms,
                );
                match result {
                    Ok(()) => Reply::Submitted,
                    // Close the virtual device after any partial or uncertain
                    // output. This worker may never accept a subsequent job.
                    Err(Refusal::Uncertain) => return Err(Refusal::Uncertain),
                    Err(reason) => Reply::Refused { reason },
                }
            }
            _ => return Err(Refusal::Protocol),
        };
        write_frame(stream, &reply).map_err(|_| Refusal::Protocol)?;
    }
}
#[allow(clippy::too_many_arguments)]
fn replace(
    stream: &UnixStream,
    completing_key: u16,
    bus: &Bus,
    system: &Bus,
    session: &Session,
    tracker: &mut Tracker,
    devices: &mut Devices,
    keymap: &mut Keymap,
    deadline: u64,
    epoch: u64,
    token: u64,
    pid: u32,
    suffix: &str,
    text: &str,
    paste: bool,
    restore: bool,
    pace: u32,
) -> Result<(), Refusal> {
    if text.len() > MAX_TEXT || !(2..=20).contains(&pace) {
        return Err(Refusal::Unsupported);
    }
    let check = || -> Result<(), Refusal> {
        let mut poll = libc::pollfd {
            fd: stream.as_raw_fd(),
            events: libc::POLLRDHUP | libc::POLLHUP | libc::POLLERR,
            revents: 0,
        };
        if unsafe { libc::poll(&mut poll, 1, 0) } < 0 || poll.revents != 0 {
            return Err(Refusal::Generation);
        }
        if now_ms() > deadline {
            return Err(Refusal::Deadline);
        }
        if !system.unlocked(&session.id) {
            return Err(Refusal::Locked);
        }
        let current = context(bus)?;
        if current.epoch != epoch || current.pid != pid {
            return Err(Refusal::Context);
        }
        Ok(())
    };
    check()?;
    devices.settle_trigger(completing_key, keymap, (now_ms() + 150).min(deadline))?;
    if keymap.modified() {
        return Err(Refusal::Modifiers);
    }
    devices.require_released().map_err(|_| Refusal::Modifiers)?;
    let keys = if paste {
        Vec::new()
    } else {
        keymap.encode(text)?
    };
    let before = tracker.snapshot(pid, epoch)?;
    before.verify(token, epoch, pid, suffix)?;
    let clipboard = if paste {
        let lease=bus.bridge(serde_json::json!({"version":VERSION,"op":"clipboardAcquire","text":text,"restore":restore}))?;
        Some(
            lease
                .get("lease")
                .and_then(|v| v.as_u64())
                .ok_or(Refusal::Clipboard)?,
        )
    } else {
        None
    };
    let mut mutated = false;
    let output = (|| {
        // A fresh AT-SPI read immediately precedes each deletion. Observe the previous
        // deletion's committed caret/suffix before submitting the next key.
        let mut previous = before.clone();
        for _ in 0..suffix.len() {
            check()?;
            devices.drain_quiet()?;
            if let Some(lease) = clipboard {
                let state = bus.bridge(
                    serde_json::json!({"version":VERSION,"op":"clipboardCheck","lease":lease}),
                )?;
                if state.get("owned").and_then(|v| v.as_bool()) != Some(true) {
                    return Err(Refusal::Clipboard);
                }
            }
            let current = tracker.snapshot(pid, epoch)?;
            if current.caret != previous.caret
                || current.suffix != previous.suffix
                || !current.safe()
                || current.token != token
            {
                return Err(Refusal::Field);
            }
            devices.require_released().map_err(|_| Refusal::Modifiers)?;
            mutated = true;
            devices.tap(14).map_err(|_| Refusal::Uncertain)?;
            let until = (now_ms() + 100).min(deadline);
            loop {
                check()?;
                devices.drain_quiet()?;
                let after = tracker.snapshot(pid, epoch)?;
                if after.erasure_observed(&previous, 1) {
                    previous = after;
                    break;
                }
                if now_ms() >= until {
                    return Err(Refusal::Uncertain);
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        check()?;
        devices.drain_quiet()?;
        let at_caret = tracker.snapshot(pid, epoch)?;
        if !at_caret.safe()
            || at_caret.token != token
            || at_caret.caret != previous.caret
            || at_caret.suffix != previous.suffix
        {
            return Err(Refusal::Field);
        }
        if paste {
            let lease = clipboard.ok_or(Refusal::Clipboard)?;
            let state = bus.bridge(
                serde_json::json!({"version":VERSION,"op":"clipboardCheck","lease":lease}),
            )?;
            if state.get("owned").and_then(|v| v.as_bool()) != Some(true) {
                return Err(Refusal::Clipboard);
            }
            devices.key(29, true).map_err(|_| Refusal::Uncertain)?;
            let result = devices.tap(47);
            let released = devices.key(29, false);
            result.and(released).map_err(|_| Refusal::Uncertain)?;
            std::thread::sleep(Duration::from_millis(100));
        } else {
            for ((code, shift), character) in keys.into_iter().zip(text.chars()) {
                check()?;
                devices.drain_quiet()?;
                devices.require_released().map_err(|_| Refusal::Modifiers)?;
                let current = tracker.snapshot(pid, epoch)?;
                if !current.safe()
                    || current.token != token
                    || current.caret != previous.caret
                    || current.suffix != previous.suffix
                {
                    return Err(Refusal::Field);
                }
                if shift {
                    devices.key(42, true).map_err(|_| Refusal::Uncertain)?
                }
                let result = devices.tap(code);
                let released = if shift {
                    devices.key(42, false)
                } else {
                    Ok(())
                };
                result.and(released).map_err(|_| Refusal::Uncertain)?;
                let expected_caret = previous.caret.checked_add(1).ok_or(Refusal::Uncertain)?;
                let mut expected_suffix = previous.suffix.clone();
                expected_suffix.push(character);
                let expected_tail: String = expected_suffix
                    .chars()
                    .rev()
                    .take(512)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();
                let until = (now_ms() + 100).min(deadline);
                loop {
                    check()?;
                    devices.drain_quiet()?;
                    let after = tracker.snapshot(pid, epoch)?;
                    if after.safe()
                        && after.token == token
                        && after.caret == expected_caret
                        && after.suffix.ends_with(&expected_tail)
                    {
                        previous = after;
                        break;
                    }
                    if now_ms() >= until {
                        return Err(Refusal::Uncertain);
                    }
                    std::thread::sleep(Duration::from_millis(u64::from(pace)));
                }
            }
        }
        Ok(())
    })();
    if let Some(lease) = clipboard {
        let _ = bus
            .bridge(serde_json::json!({"version":VERSION,"op":"clipboardRelease","lease":lease}));
    }
    if mutated && output.is_err() {
        Err(Refusal::Uncertain)
    } else {
        output
    }
}

pub(super) fn current_session_unlocked() -> bool {
    let Ok(bus) = Bus::open(true) else {
        return false;
    };
    let mut id = ptr::null_mut();
    unsafe {
        if sd_pid_get_session(libc::getpid(), &mut id) < 0 || id.is_null() {
            return false;
        }
        let mut uid = 0;
        let mut class = ptr::null_mut();
        let mut seat = ptr::null_mut();
        let ok = sd_session_get_uid(id, &mut uid) >= 0
            && uid == libc::geteuid()
            && uid != 0
            && sd_session_is_active(id) > 0
            && sd_session_is_remote(id) == 0
            && sd_session_get_class(id, &mut class) >= 0
            && !class.is_null()
            && CStr::from_ptr(class) == c"user"
            && sd_session_get_seat(id, &mut seat) >= 0
            && !seat.is_null()
            && CStr::from_ptr(seat) == c"seat0"
            && bus.unlocked(&CStr::from_ptr(id).to_string_lossy());
        libc::free(class.cast());
        libc::free(seat.cast());
        libc::free(id.cast());
        ok
    }
}
