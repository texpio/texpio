//! Small audited ABI surface. Create/Copy results are owned by `Owned`.
#![allow(non_snake_case)]
use std::ffi::{c_char, c_void};
pub type Ref = *mut c_void;
pub type Event = Ref;
pub type Callback = unsafe extern "C" fn(Ref, u32, Event, Ref) -> Event;
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    pub static kCFRunLoopDefaultMode: Ref;
    pub fn CFRelease(value: Ref);
    pub fn CFEqual(a: Ref, b: Ref) -> bool;
    pub fn CFGetTypeID(value: Ref) -> usize;
    pub fn CFStringGetTypeID() -> usize;
    pub fn CFStringCreateWithBytes(
        allocator: Ref,
        bytes: *const u8,
        length: isize,
        encoding: u32,
        external: bool,
    ) -> Ref;
    pub fn CFStringGetCString(value: Ref, bytes: *mut c_char, size: isize, encoding: u32) -> bool;
    pub fn CFRunLoopGetCurrent() -> Ref;
    pub fn CFRunLoopAddSource(runloop: Ref, source: Ref, mode: Ref);
    pub fn CFRunLoopRemoveSource(runloop: Ref, source: Ref, mode: Ref);
    pub fn CFRunLoopRunInMode(mode: Ref, seconds: f64, return_after: bool) -> i32;
    pub fn CFMachPortCreateRunLoopSource(allocator: Ref, port: Ref, order: isize) -> Ref;
    pub fn CFMachPortInvalidate(port: Ref);
    pub fn CFDataGetTypeID() -> usize;
    pub fn CFDataGetBytePtr(data: Ref) -> *const u8;
    pub fn CFBooleanGetTypeID() -> usize;
    pub fn CFBooleanGetValue(value: Ref) -> bool;
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    pub fn AXIsProcessTrusted() -> bool;
    pub fn AXIsProcessTrustedWithOptions(options: Ref) -> bool;
    pub static kAXTrustedCheckOptionPrompt: Ref;
    pub fn AXUIElementCreateSystemWide() -> Ref;
    pub fn AXUIElementCopyAttributeValue(element: Ref, attribute: Ref, value: *mut Ref) -> i32;
    pub fn AXUIElementGetPid(element: Ref, pid: *mut i32) -> i32;
    pub fn AXUIElementSetMessagingTimeout(element: Ref, seconds: f32) -> i32;
    pub fn AXUIElementIsAttributeSettable(element: Ref, attribute: Ref, result: *mut bool) -> i32;
    pub fn AXValueGetTypeID() -> usize;
    pub fn AXValueGetType(value: Ref) -> u32;
    pub fn AXValueGetValue(value: Ref, kind: u32, result: *mut c_void) -> bool;
    pub fn CGPreflightListenEventAccess() -> bool;
    pub fn CGPreflightPostEventAccess() -> bool;
    pub fn CGRequestListenEventAccess() -> bool;
    pub fn CGRequestPostEventAccess() -> bool;
    pub fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        mask: u64,
        callback: Callback,
        user: Ref,
    ) -> Ref;
    pub fn CGEventTapEnable(tap: Ref, enable: bool);
    pub fn CGEventTapIsEnabled(tap: Ref) -> bool;
    pub fn CGEventGetIntegerValueField(event: Event, field: u32) -> i64;
    pub fn CGEventSetIntegerValueField(event: Event, field: u32, value: i64);
    pub fn CGEventGetFlags(event: Event) -> u64;
    pub fn CGEventSourceFlagsState(state: i32) -> u64;
    pub fn CGEventKeyboardSetUnicodeString(event: Event, count: usize, bytes: *const u16);
    pub fn CGEventCreateKeyboardEvent(source: Ref, key: u16, down: bool) -> Event;
    pub fn CGEventSetFlags(event: Event, flags: u64);
    pub fn CGEventPost(tap: u32, event: Event);
}
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    pub fn IsSecureEventInputEnabled() -> bool;
    pub fn TISCopyCurrentKeyboardInputSource() -> Ref;
    pub fn TISGetInputSourceProperty(source: Ref, key: Ref) -> Ref;
    pub static kTISPropertyInputSourceID: Ref;
    pub static kTISPropertyInputSourceType: Ref;
    pub static kTISPropertyInputSourceIsASCIICapable: Ref;
    pub static kTISPropertyUnicodeKeyLayoutData: Ref;
    pub static kTISTypeKeyboardLayout: Ref;
    pub fn LMGetKbdType() -> u8;
    pub fn UCKeyTranslate(
        layout: *const c_void,
        keycode: u16,
        action: u16,
        modifiers: u32,
        keyboard_type: u32,
        options: u32,
        dead_key_state: *mut u32,
        max_length: usize,
        actual_length: *mut usize,
        unicode_string: *mut u16,
    ) -> i32;
}
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const c_char) -> Ref;
    fn sel_registerName(name: *const c_char) -> Ref;
    fn objc_msgSend();
    pub fn objc_autoreleasePoolPush() -> Ref;
    pub fn objc_autoreleasePoolPop(pool: Ref);
}
#[derive(Debug)]
pub struct Owned(pub Ref);
pub struct Tap(pub Owned);
impl Drop for Tap {
    fn drop(&mut self) {
        unsafe { CFMachPortInvalidate(self.0.0) };
    }
}
// CF AX references are immutable identities. All queries are bounded on the worker.
unsafe impl Send for Owned {}
unsafe impl Sync for Owned {}
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0) };
        }
    }
}
pub fn owned(value: Ref) -> Option<Owned> {
    (!value.is_null()).then_some(Owned(value))
}
pub fn string(value: &str) -> Owned {
    Owned(unsafe {
        CFStringCreateWithBytes(
            std::ptr::null_mut(),
            value.as_ptr(),
            value.len() as isize,
            0x08000100,
            false,
        )
    })
}
pub fn text(value: Ref) -> Option<String> {
    if value.is_null() || unsafe { CFGetTypeID(value) != CFStringGetTypeID() } {
        return None;
    }
    let mut bytes = [0i8; 1024];
    if !unsafe { CFStringGetCString(value, bytes.as_mut_ptr(), bytes.len() as isize, 0x08000100) } {
        return None;
    }
    Some(
        unsafe { std::ffi::CStr::from_ptr(bytes.as_ptr()) }
            .to_string_lossy()
            .into_owned(),
    )
}
pub fn attribute(element: Ref, name: &str) -> Option<Owned> {
    let key = string(name);
    let mut value = std::ptr::null_mut();
    (unsafe { AXUIElementCopyAttributeValue(element, key.0, &mut value) } == 0)
        .then(|| owned(value))
        .flatten()
}
pub fn class(name: &std::ffi::CStr) -> Ref {
    unsafe { objc_getClass(name.as_ptr()) }
}
fn selector(name: &std::ffi::CStr) -> Ref {
    unsafe { sel_registerName(name.as_ptr()) }
}
pub unsafe fn send0<R>(object: Ref, name: &std::ffi::CStr) -> R {
    let fun: unsafe extern "C" fn(Ref, Ref) -> R =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { fun(object, selector(name)) }
}
pub unsafe fn send1<A, R>(object: Ref, name: &std::ffi::CStr, a: A) -> R {
    let fun: unsafe extern "C" fn(Ref, Ref, A) -> R =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { fun(object, selector(name), a) }
}
pub unsafe fn send2<A, B, R>(object: Ref, name: &std::ffi::CStr, a: A, b: B) -> R {
    let fun: unsafe extern "C" fn(Ref, Ref, A, B) -> R =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { fun(object, selector(name), a, b) }
}
pub struct Pool(Ref);
impl Pool {
    pub fn new() -> Self {
        Self(unsafe { objc_autoreleasePoolPush() })
    }
}
impl Drop for Pool {
    fn drop(&mut self) {
        unsafe { objc_autoreleasePoolPop(self.0) };
    }
}
