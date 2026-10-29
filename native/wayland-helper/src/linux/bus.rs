use crate::Refusal;
use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    ptr,
};
#[repr(C)]
struct Error {
    name: *const c_char,
    message: *const c_char,
    need_free: c_int,
}
#[link(name = "systemd")]
unsafe extern "C" {
    fn sd_bus_open_system(bus: *mut *mut c_void) -> c_int;
    fn sd_bus_open_user(bus: *mut *mut c_void) -> c_int;
    fn sd_bus_unref(bus: *mut c_void) -> *mut c_void;
    fn sd_bus_set_method_call_timeout(bus: *mut c_void, usec: u64) -> c_int;
    fn sd_bus_call_method(
        bus: *mut c_void,
        destination: *const c_char,
        path: *const c_char,
        interface: *const c_char,
        member: *const c_char,
        error: *mut Error,
        reply: *mut *mut c_void,
        types: *const c_char,
        ...
    ) -> c_int;
    fn sd_bus_message_read(message: *mut c_void, types: *const c_char, ...) -> c_int;
    fn sd_bus_message_unref(message: *mut c_void) -> *mut c_void;
    fn sd_bus_error_free(error: *mut Error);
    fn sd_bus_path_encode(
        prefix: *const c_char,
        external: *const c_char,
        path: *mut *mut c_char,
    ) -> c_int;
    fn sd_bus_get_property_trivial(
        bus: *mut c_void,
        destination: *const c_char,
        path: *const c_char,
        interface: *const c_char,
        member: *const c_char,
        error: *mut Error,
        type_: c_char,
        ret: *mut c_void,
    ) -> c_int;
}
pub struct Bus(*mut c_void);
impl Bus {
    pub fn open(system: bool) -> Result<Self, Refusal> {
        let mut p = ptr::null_mut();
        let result = unsafe {
            if system {
                sd_bus_open_system(&mut p)
            } else {
                sd_bus_open_user(&mut p)
            }
        };
        if result < 0 || p.is_null() {
            return Err(Refusal::BridgeMissing);
        }
        unsafe {
            sd_bus_set_method_call_timeout(p, 200_000);
        }
        Ok(Self(p))
    }
    pub fn unlocked(&self, session: &str) -> bool {
        let Ok(id) = CString::new(session) else {
            return false;
        };
        let mut path = ptr::null_mut();
        let mut locked = 1i32;
        let mut error: Error = unsafe { std::mem::zeroed() };
        let result = unsafe {
            if sd_bus_path_encode(
                c"/org/freedesktop/login1/session".as_ptr(),
                id.as_ptr(),
                &mut path,
            ) < 0
            {
                return false;
            }
            let result = sd_bus_get_property_trivial(
                self.0,
                c"org.freedesktop.login1".as_ptr(),
                path,
                c"org.freedesktop.login1.Session".as_ptr(),
                c"LockedHint".as_ptr(),
                &mut error,
                b'b' as c_char,
                (&mut locked as *mut i32).cast(),
            );
            libc::free(path.cast());
            sd_bus_error_free(&mut error);
            result
        };
        result >= 0 && locked == 0
    }
    pub fn bridge(&self, request: serde_json::Value) -> Result<serde_json::Value, Refusal> {
        let request = CString::new(request.to_string()).map_err(|_| Refusal::Protocol)?;
        if request.as_bytes().len() > crate::MAX_FRAME {
            return Err(Refusal::Protocol);
        }
        let mut reply = ptr::null_mut();
        let mut error: Error = unsafe { std::mem::zeroed() };
        let result = unsafe {
            sd_bus_call_method(
                self.0,
                c"org.texpio.GnomeBridge".as_ptr(),
                c"/org/texpio/GnomeBridge".as_ptr(),
                c"org.texpio.GnomeBridge1".as_ptr(),
                c"Exchange".as_ptr(),
                &mut error,
                &mut reply,
                c"s".as_ptr(),
                request.as_ptr(),
            )
        };
        let authentication_unavailable = !error.name.is_null()
            && unsafe { CStr::from_ptr(error.name) }
                == c"org.texpio.Error.AuthenticationUnavailable";
        unsafe {
            sd_bus_error_free(&mut error);
        }
        if result < 0 || reply.is_null() {
            return Err(if authentication_unavailable {
                Refusal::BridgeAuthentication
            } else {
                Refusal::BridgeMissing
            });
        }
        let mut text: *const c_char = ptr::null();
        let result = unsafe { sd_bus_message_read(reply, c"s".as_ptr(), &mut text) };
        let value = if result > 0 && !text.is_null() {
            let bytes = unsafe { CStr::from_ptr(text) }.to_bytes();
            if bytes.len() <= crate::MAX_FRAME {
                serde_json::from_slice(bytes).map_err(|_| Refusal::Protocol)
            } else {
                Err(Refusal::Protocol)
            }
        } else {
            Err(Refusal::Protocol)
        };
        unsafe {
            sd_bus_message_unref(reply);
        }
        value
    }
}
impl Drop for Bus {
    fn drop(&mut self) {
        unsafe {
            sd_bus_unref(self.0);
        }
    }
}
