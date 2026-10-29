use crate::Refusal;
use std::{
    ffi::{CString, c_char, c_int, c_void},
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{fs::MetadataExt, fs::OpenOptionsExt},
    },
    path::PathBuf,
};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Event {
    time: libc::timeval,
    pub kind: u16,
    pub code: u16,
    pub value: i32,
}
#[repr(C)]
struct InputId {
    bus: u16,
    vendor: u16,
    product: u16,
    version: u16,
}
#[repr(C)]
struct Setup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}
#[repr(C)]
struct Names {
    rules: *const c_char,
    model: *const c_char,
    layout: *const c_char,
    variant: *const c_char,
    options: *const c_char,
}
#[link(name = "xkbcommon")]
unsafe extern "C" {
    fn xkb_context_new(flags: u32) -> *mut c_void;
    fn xkb_context_unref(p: *mut c_void);
    fn xkb_keymap_new_from_names(c: *mut c_void, n: *const Names, f: u32) -> *mut c_void;
    fn xkb_keymap_unref(p: *mut c_void);
    fn xkb_state_new(k: *mut c_void) -> *mut c_void;
    fn xkb_state_unref(p: *mut c_void);
    fn xkb_state_update_key(s: *mut c_void, k: u32, d: u32) -> u32;
    fn xkb_state_key_get_utf32(s: *mut c_void, k: u32) -> u32;
    fn xkb_state_mod_name_is_active(s: *mut c_void, n: *const c_char, c: u32) -> c_int;
}
pub struct Keymap {
    context: *mut c_void,
    map: *mut c_void,
    state: *mut c_void,
}
impl Keymap {
    pub fn new(layout: &str, options: &str) -> Result<Self, Refusal> {
        if layout.len() > 128
            || options.len() > 512
            || !layout
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"+_-".contains(&b))
            || !options
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b",:_-".contains(&b))
        {
            return Err(Refusal::Context);
        }
        let (layout, variant) = layout.split_once('+').unwrap_or((layout, ""));
        let l = CString::new(layout).map_err(|_| Refusal::Context)?;
        let v = CString::new(variant).map_err(|_| Refusal::Context)?;
        let o = CString::new(options).map_err(|_| Refusal::Context)?;
        unsafe {
            let context = xkb_context_new(0);
            if context.is_null() {
                return Err(Refusal::Context);
            }
            let names = Names {
                rules: c"evdev".as_ptr(),
                model: c"pc105".as_ptr(),
                layout: l.as_ptr(),
                variant: v.as_ptr(),
                options: o.as_ptr(),
            };
            let map = xkb_keymap_new_from_names(context, &names, 0);
            if map.is_null() {
                xkb_context_unref(context);
                return Err(Refusal::Context);
            }
            let state = xkb_state_new(map);
            if state.is_null() {
                xkb_keymap_unref(map);
                xkb_context_unref(context);
                return Err(Refusal::Context);
            }
            Ok(Self {
                context,
                map,
                state,
            })
        }
    }
    pub fn update(&mut self, e: Event) -> Option<char> {
        unsafe {
            if e.kind != 1 {
                return None;
            }
            if e.value == 0 {
                xkb_state_update_key(self.state, u32::from(e.code) + 8, 0);
                return None;
            }
            if e.value == 1 {
                xkb_state_update_key(self.state, u32::from(e.code) + 8, 1);
            }
            char::from_u32(xkb_state_key_get_utf32(self.state, u32::from(e.code) + 8))
        }
    }
    pub fn modified(&self) -> bool {
        unsafe {
            [
                c"Shift", c"Control", c"Mod1", c"Mod4", c"Mod5", c"Lock", c"Mod2",
            ]
            .iter()
            .any(|n| xkb_state_mod_name_is_active(self.state, n.as_ptr(), 8) > 0)
        }
    }
    /// Prepare every scalar before the first erase; no keymap mutation or Unicode fallback.
    pub fn encode(&self, text: &str) -> Result<Vec<(u16, bool)>, Refusal> {
        let mut result = Vec::new();
        for ch in text.chars() {
            if ch.is_control() {
                return Err(Refusal::Unsupported);
            }
            let mut found = None;
            for shift in [false, true] {
                let state = unsafe { xkb_state_new(self.map) };
                if state.is_null() {
                    return Err(Refusal::Input);
                }
                if shift {
                    unsafe {
                        xkb_state_update_key(state, 50, 1);
                    }
                }
                for code in 1..=255u16 {
                    if unsafe { xkb_state_key_get_utf32(state, u32::from(code) + 8) } == ch as u32 {
                        found = Some((code, shift));
                        break;
                    }
                }
                unsafe {
                    xkb_state_unref(state);
                }
                if found.is_some() {
                    break;
                }
            }
            result.push(found.ok_or(Refusal::Unsupported)?);
        }
        Ok(result)
    }
}
impl Drop for Keymap {
    fn drop(&mut self) {
        unsafe {
            xkb_state_unref(self.state);
            xkb_keymap_unref(self.map);
            xkb_context_unref(self.context);
        }
    }
}
pub fn device_paths() -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in fs::read_dir("/dev/input")? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("event") || !name[5..].bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let sys = fs::canonicalize(format!("/sys/class/input/{name}"))?;
        if sys.to_string_lossy().contains("/devices/virtual/") {
            continue;
        }
        let metadata = fs::metadata(&path)?;
        let dev = metadata.rdev();
        let props = fs::read_to_string(format!(
            "/run/udev/data/c{}:{}",
            libc::major(dev),
            libc::minor(dev)
        ))?;
        if !crate::approved_device(&sys.to_string_lossy(), &props) {
            continue;
        }
        paths.push(path);
    }
    paths.sort();
    if paths.is_empty() {
        return Err(io::ErrorKind::NotFound.into());
    }
    Ok(paths)
}
pub struct Devices {
    pub inputs: Vec<File>,
    pub paths: Vec<PathBuf>,
    output: crate::SyntheticKeys<UInput>,
}
struct UInput(File);
impl UInput {
    fn write(&mut self, kind: u16, code: u16, value: i32) -> io::Result<()> {
        let event = Event {
            time: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            kind,
            code,
            value,
        };
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&event as *const Event).cast::<u8>(),
                std::mem::size_of::<Event>(),
            )
        };
        self.0.write_all(bytes)
    }
}
impl crate::KeyOutput for UInput {
    fn event(&mut self, code: u16, down: bool) -> io::Result<()> {
        self.write(1, code, i32::from(down))
    }
    fn sync(&mut self) -> io::Result<()> {
        self.write(0, 0, 0)
    }
}
impl Devices {
    pub fn drain_quiet(&mut self) -> Result<(), Refusal> {
        let mut gate = crate::ObservationGate::quiet();
        for _ in 0..4096 {
            let Some(event) = self.next().map_err(|_| Refusal::Input)? else {
                return Ok(());
            };
            if !gate.accept(event.kind, event.code, event.value) {
                return Err(Refusal::Input);
            }
        }
        Err(Refusal::Input)
    }
    pub fn settle_trigger(
        &mut self,
        code: u16,
        keymap: &mut Keymap,
        deadline: u64,
    ) -> Result<(), Refusal> {
        let mut gate = crate::ObservationGate::trigger_release(code);
        loop {
            if super::now_ms() >= deadline {
                return Err(Refusal::Deadline);
            }
            if let Some(event) = self.next().map_err(|_| Refusal::Input)? {
                if !gate.accept(event.kind, event.code, event.value) {
                    return Err(Refusal::Input);
                }
                keymap.update(event);
            } else if self.require_released().is_ok() {
                return Ok(());
            } else {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
        }
    }
    pub fn open() -> io::Result<Self> {
        let paths = device_paths()?;
        let inputs = paths
            .iter()
            .map(|p| {
                OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                    .open(p)
            })
            .collect::<io::Result<Vec<_>>>()?;
        let output = OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open("/dev/uinput")?;
        let fd = output.as_raw_fd();
        unsafe {
            if libc::ioctl(fd, 0x40045564u64, 1) < 0 {
                return Err(io::Error::last_os_error());
            }
            for key in 1..=255 {
                if libc::ioctl(fd, 0x40045565u64, key) < 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            let mut setup = Setup {
                id: InputId {
                    bus: 3,
                    vendor: 0x5458,
                    product: 1,
                    version: 1,
                },
                name: [0; 80],
                ff_effects_max: 0,
            };
            let name = b"Texpio virtual keyboard";
            setup.name[..name.len()].copy_from_slice(name);
            if libc::ioctl(fd, 0x405c5503u64, &setup) < 0 || libc::ioctl(fd, 0x5501u64) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        let result = Self {
            inputs,
            paths,
            output: crate::SyntheticKeys::new(UInput(output)),
        };
        result.require_released()?;
        Ok(result)
    }
    pub fn require_released(&self) -> io::Result<()> {
        for file in &self.inputs {
            let mut keys = [0u8; 96];
            if unsafe { libc::ioctl(file.as_raw_fd(), 0x80604518u64, keys.as_mut_ptr()) } < 0 {
                return Err(io::Error::last_os_error());
            }
            if keys.iter().any(|b| *b != 0) {
                return Err(io::ErrorKind::WouldBlock.into());
            }
            let mut leds = [0u8; 8];
            if unsafe { libc::ioctl(file.as_raw_fd(), 0x80084519u64, leds.as_mut_ptr()) } < 0 {
                return Err(io::Error::last_os_error());
            }
            if leds[0] & 7 != 0 {
                return Err(io::ErrorKind::WouldBlock.into());
            }
        }
        Ok(())
    }
    pub fn next(&mut self) -> io::Result<Option<Event>> {
        for file in &mut self.inputs {
            let mut e = std::mem::MaybeUninit::<Event>::zeroed();
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    e.as_mut_ptr().cast::<u8>(),
                    std::mem::size_of::<Event>(),
                )
            };
            match file.read(bytes) {
                Ok(n) if n == bytes.len() => return Ok(Some(unsafe { e.assume_init() })),
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                Ok(_) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Err(err) => return Err(err),
            }
        }
        Ok(None)
    }
    pub fn key(&mut self, code: u16, down: bool) -> io::Result<()> {
        self.output.key(code, down)
    }
    pub fn tap(&mut self, code: u16) -> io::Result<()> {
        self.output.tap(code)
    }
}
// The kernel destroys the virtual device when the last output descriptor closes.
// Do not issue UI_DEV_DESTROY from the broker parent after fork.
