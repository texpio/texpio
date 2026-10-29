//! Explicit non-restoring native paste. AppKit has no atomic compare-and-clear.
use super::{
    ffi::*,
    policy::{lease_matches, mode_supported},
};
use crate::product::effects::ProductError;
const OWNER_TYPE: &str = "net.texpio.expansion-owner";
pub struct Lease {
    board: Ref,
    owned: isize,
    token: Vec<u8>,
}
impl Lease {
    pub fn acquire(
        text_value: &str,
        restore: bool,
        mut check: impl FnMut() -> Result<(), ProductError>,
    ) -> Result<Self, ProductError> {
        // Defense in depth: restoration is unavailable even if a caller bypasses
        // runtime/admission guards. No previous clipboard data is read or restored.
        if !mode_supported(true, restore) {
            return Err(ProductError::Unavailable);
        }
        check()?;
        unsafe {
            let board: Ref = send0(class(c"NSPasteboard"), c"generalPasteboard");
            if board.is_null() {
                return Err(ProductError::Clipboard);
            }
            let token = uuid::Uuid::new_v4().to_string().into_bytes();
            let allocated: Ref = send0(class(c"NSPasteboardItem"), c"alloc");
            let item: Ref = send0(allocated, c"init");
            if item.is_null() {
                return Err(ProductError::Clipboard);
            }
            let mut prepared = true;
            for (kind, bytes) in [
                ("public.utf8-plain-text", text_value.as_bytes()),
                (OWNER_TYPE, token.as_slice()),
            ] {
                let kind = string(kind);
                let data: Ref = send2(
                    class(c"NSData"),
                    c"dataWithBytes:length:",
                    bytes.as_ptr(),
                    bytes.len(),
                );
                prepared &= send2::<_, _, bool>(item, c"setData:forType:", data, kind.0);
            }
            let array: Ref = send0(class(c"NSMutableArray"), c"array");
            let _: () = send1(array, c"addObject:", item);
            let _: () = send0(item, c"release");
            if !prepared {
                return Err(ProductError::Clipboard);
            }
            check()?;
            // This is an explicitly requested destructive replacement. AppKit
            // cannot make a restoration compare-and-clear atomic, so no restore
            // path or fallback write is offered here.
            let _: isize = send0(board, c"clearContents");
            if !send1::<_, bool>(board, c"writeObjects:", array) {
                return Err(ProductError::Clipboard);
            }
            let lease = Self {
                board,
                owned: send0(board, c"changeCount"),
                token,
            };
            if !lease.owns() {
                return Err(ProductError::Clipboard);
            }
            Ok(lease)
        }
    }
    pub fn owns(&self) -> bool {
        unsafe {
            let marker = string(OWNER_TYPE);
            let data: Ref = send1(self.board, c"dataForType:", marker.0);
            let marker_matches = if data.is_null() {
                false
            } else {
                let count: usize = send0(data, c"length");
                if count != self.token.len() {
                    false
                } else {
                    let ptr: *const u8 = send0(data, c"bytes");
                    !ptr.is_null() && std::slice::from_raw_parts(ptr, count) == self.token
                }
            };
            lease_matches(
                send0(self.board, c"changeCount"),
                self.owned,
                marker_matches,
            )
        }
    }
}
