//! AT-SPI reads are bounded and never treated as composition evidence.
use crate::{Composition, FieldEvidence, Refusal};
use std::{
    ffi::{CStr, c_char, c_int, c_void},
    ptr,
};
#[link(name = "atspi")]
unsafe extern "C" {
    fn atspi_init() -> c_int;
    fn atspi_is_initialized() -> c_int;
    fn atspi_set_timeout(normal: c_int, startup: c_int);
    fn atspi_get_desktop(index: c_int) -> *mut c_void;
    fn atspi_accessible_get_child_count(o: *mut c_void, e: *mut *mut c_void) -> c_int;
    fn atspi_accessible_get_child_at_index(
        o: *mut c_void,
        i: c_int,
        e: *mut *mut c_void,
    ) -> *mut c_void;
    fn atspi_accessible_get_process_id(o: *mut c_void, e: *mut *mut c_void) -> u32;
    fn atspi_accessible_get_state_set(o: *mut c_void) -> *mut c_void;
    fn atspi_state_set_contains(o: *mut c_void, state: c_int) -> c_int;
    fn atspi_accessible_get_role_name(o: *mut c_void, e: *mut *mut c_void) -> *mut c_char;
    fn atspi_accessible_get_text_iface(o: *mut c_void) -> *mut c_void;
    fn atspi_accessible_clear_cache(o: *mut c_void);
    fn atspi_text_get_caret_offset(o: *mut c_void, e: *mut *mut c_void) -> c_int;
    fn atspi_text_get_n_selections(o: *mut c_void, e: *mut *mut c_void) -> c_int;
    fn atspi_text_get_text(
        o: *mut c_void,
        start: c_int,
        end: c_int,
        e: *mut *mut c_void,
    ) -> *mut c_char;
}
#[link(name = "gobject-2.0")]
unsafe extern "C" {
    fn g_object_unref(o: *mut c_void);
}
#[link(name = "glib-2.0")]
unsafe extern "C" {
    fn g_free(o: *mut c_void);
    fn g_error_free(o: *mut c_void);
}
struct Object(*mut c_void);
impl Drop for Object {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { g_object_unref(self.0) }
        }
    }
}
fn checked<T>(call: impl FnOnce(*mut *mut c_void) -> T) -> Result<T, Refusal> {
    let mut error = ptr::null_mut();
    let result = call(&mut error);
    if error.is_null() {
        Ok(result)
    } else {
        unsafe {
            g_error_free(error);
        }
        Err(Refusal::Field)
    }
}
fn string(p: *mut c_char) -> Result<String, Refusal> {
    if p.is_null() {
        return Err(Refusal::Field);
    }
    let bytes = unsafe { CStr::from_ptr(p) }.to_bytes();
    let result = if bytes.len() <= 4096 {
        String::from_utf8(bytes.to_vec()).map_err(|_| Refusal::Field)
    } else {
        Err(Refusal::Field)
    };
    unsafe {
        g_free(p.cast());
    }
    result
}
pub struct Tracker {
    current: Option<Object>,
    token: u64,
}
impl Tracker {
    pub fn new() -> Result<Self, Refusal> {
        unsafe {
            atspi_set_timeout(25, 100);
            if !matches!(atspi_init(), 0 | 1) || atspi_is_initialized() == 0 {
                return Err(Refusal::Field);
            }
        }
        Ok(Self {
            current: None,
            token: 0,
        })
    }
    pub fn snapshot(&mut self, pid: u32, epoch: u64) -> Result<FieldEvidence, Refusal> {
        let start = super::now_ms();
        let desktop = Object(unsafe { atspi_get_desktop(0) });
        if desktop.0.is_null() {
            return Err(Refusal::Field);
        }
        let mut queue = vec![(desktop, 0)];
        let mut found = None;
        let mut visited = 0;
        while let Some((obj, depth)) = queue.pop() {
            visited += 1;
            if visited > 256 || super::now_ms().saturating_sub(start) > 150 {
                return Err(Refusal::Field);
            }
            unsafe { atspi_accessible_clear_cache(obj.0) }
            let object_pid = checked(|e| unsafe { atspi_accessible_get_process_id(obj.0, e) })?;
            let state = Object(unsafe { atspi_accessible_get_state_set(obj.0) });
            if state.0.is_null() {
                return Err(Refusal::Field);
            }
            if object_pid == pid && unsafe { atspi_state_set_contains(state.0, 12) } > 0 {
                if found.is_some() {
                    return Err(Refusal::Field);
                }
                found = Some(obj);
                continue;
            }
            if !crate::traverse_children(depth, object_pid, pid) {
                continue;
            }
            let count = checked(|e| unsafe { atspi_accessible_get_child_count(obj.0, e) })?;
            if !(0..=256).contains(&count) {
                return Err(Refusal::Field);
            }
            for i in 0..count {
                let child =
                    checked(|e| unsafe { atspi_accessible_get_child_at_index(obj.0, i, e) })?;
                if child.is_null() {
                    return Err(Refusal::Field);
                }
                queue.push((Object(child), depth + 1));
            }
        }
        let obj = found.ok_or(Refusal::Field)?;
        if self.current.as_ref().map(|o| o.0) != Some(obj.0) {
            self.token = self.token.checked_add(1).ok_or(Refusal::Field)?;
            self.current = Some(obj);
        }
        let obj = self.current.as_ref().ok_or(Refusal::Field)?.0;
        let state = Object(unsafe { atspi_accessible_get_state_set(obj) });
        if state.0.is_null() {
            return Err(Refusal::Field);
        }
        let role = string(checked(|e| unsafe {
            atspi_accessible_get_role_name(obj, e)
        })?)?;
        let editable = unsafe { atspi_state_set_contains(state.0, 7) } > 0;
        let protected = role == "password text" || !["text", "entry"].contains(&role.as_str());
        if protected || !editable {
            return Err(Refusal::Field);
        }
        let text = Object(unsafe { atspi_accessible_get_text_iface(obj) });
        if text.0.is_null() {
            return Err(Refusal::Field);
        }
        let caret = checked(|e| unsafe { atspi_text_get_caret_offset(text.0, e) })?;
        if caret < 0 {
            return Err(Refusal::Field);
        }
        let selections = checked(|e| unsafe { atspi_text_get_n_selections(text.0, e) })?;
        let suffix = string(checked(|e| unsafe {
            atspi_text_get_text(text.0, caret.saturating_sub(512).max(0), caret, e)
        })?)?;
        // AT-SPI Text has no generic preedit/compose-state contract. Shell's private
        // InputMethod observes its own IBus context, not this client's context. A
        // future qualified per-toolkit provider must supply positive Idle evidence;
        // neither an XKB source nor an unchanged caret establishes that fact.
        Ok(FieldEvidence {
            token: self.token,
            context_epoch: epoch,
            pid,
            caret,
            suffix,
            focused: true,
            editable,
            protected,
            composition: Composition::Unknown,
            selections,
        })
    }
}
