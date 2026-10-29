//! Xlib's XKB lookup implements server key types, active groups and consumed modifiers.
//! No keymap is ever changed. This initial profile recognizes ASCII and refuses IMEs.
use crate::product::effects::ProductError;
use std::ffi::c_void;
#[cfg(target_os = "linux")]
use std::ffi::{c_char, c_int, c_uint, c_ulong};

#[cfg(target_os = "linux")]
#[link(name = "X11")]
unsafe extern "C" {
    fn XOpenDisplay(name: *const c_char) -> *mut c_void;
    fn XCloseDisplay(display: *mut c_void) -> c_int;
    fn XkbLookupKeySym(
        display: *mut c_void,
        code: u8,
        state: c_uint,
        consumed: *mut c_uint,
        symbol: *mut c_ulong,
    ) -> c_int;
}

pub(super) struct Keyboard(*mut c_void);
impl Keyboard {
    pub fn open() -> Result<Self, ProductError> {
        #[cfg(not(target_os = "linux"))]
        {
            return Err(ProductError::Unavailable);
        }
        #[cfg(target_os = "linux")]
        {
            // SAFETY: Xlib connection is owned by and used only on the recording thread.
            let display = unsafe { XOpenDisplay(std::ptr::null()) };
            if display.is_null() {
                Err(ProductError::Unavailable)
            } else {
                Ok(Self(display))
            }
        }
    }
    pub fn symbol(&self, key: u8, state: u16) -> Option<u64> {
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (key, state);
            return None;
        }
        #[cfg(target_os = "linux")]
        {
            let mut consumed = 0;
            let mut symbol = 0;
            // SAFETY: output pointers are valid, display is live and thread-confined.
            let ok =
                unsafe { XkbLookupKeySym(self.0, key, state.into(), &mut consumed, &mut symbol) };
            (ok != 0).then_some(symbol)
        }
    }
}
impl Drop for Keyboard {
    fn drop(&mut self) {
        // SAFETY: this is the unique owner and no calls survive this thread.
        #[cfg(target_os = "linux")]
        unsafe {
            XCloseDisplay(self.0);
        }
    }
}
