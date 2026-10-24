//! Clipboard ownership is checked while it is locked, including failure cleanup.
use super::effects::ProductError;
use std::{
    thread,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
        System::{DataExchange::*, LibraryLoader::GetModuleHandleW, Memory::*},
        UI::WindowsAndMessaging::*,
    },
    core::w,
};

pub struct PasteBoard {
    owner: HWND,
}
fn file_bytes(paths: &[std::path::PathBuf]) -> Result<Vec<u8>, ProductError> {
    use std::os::windows::ffi::OsStrExt;
    if paths.is_empty() || paths.len() > 100 || paths.iter().any(|path| !path.is_absolute()) {
        return Err(ProductError::Clipboard);
    }
    let mut units = Vec::new();
    for path in paths {
        let path: Vec<_> = path.as_os_str().encode_wide().collect();
        if path.contains(&0) {
            return Err(ProductError::Clipboard);
        }
        units.extend(path);
        units.push(0);
    }
    units.push(0);
    if units.len() > 65536 {
        return Err(ProductError::Clipboard);
    }
    let mut bytes = vec![0; 20];
    bytes[..4].copy_from_slice(&20u32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1u32.to_le_bytes());
    bytes.extend(units.into_iter().flat_map(u16::to_le_bytes));
    Ok(bytes)
}
struct Locked;
impl Drop for Locked {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
impl PasteBoard {
    pub fn new() -> Result<Self, ProductError> {
        // SAFETY: an invisible message-only window owned by this worker/thread.
        let owner = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!(""),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(
                    GetModuleHandleW(None)
                        .map_err(|_| ProductError::Clipboard)?
                        .into(),
                ),
                None,
            )
        }
        .map_err(|_| ProductError::Clipboard)?;
        Ok(Self { owner })
    }
    fn lock(&self) -> Result<Locked, ProductError> {
        let deadline = Instant::now() + Duration::from_millis(100);
        loop {
            // SAFETY: owner remains alive on this thread through the lock.
            if unsafe { OpenClipboard(Some(self.owner)) }.is_ok() {
                return Ok(Locked);
            }
            if Instant::now() >= deadline {
                return Err(ProductError::Clipboard);
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
    pub fn text(&self) -> Result<Option<String>, ProductError> {
        let _locked = self.lock()?;
        // SAFETY: clipboard is locked, handle lifetime ends before unlocking.
        unsafe {
            let Ok(handle) = GetClipboardData(13) else {
                return Ok(None);
            };
            let memory = HGLOBAL(handle.0);
            let size = GlobalSize(memory);
            if size > 8 * 1024 * 1024 || !size.is_multiple_of(2) {
                return Err(ProductError::Clipboard);
            }
            let pointer = GlobalLock(memory);
            if pointer.is_null() {
                return Err(ProductError::Clipboard);
            }
            let units = std::slice::from_raw_parts(pointer.cast::<u16>(), size / 2);
            let count = units
                .iter()
                .position(|unit| *unit == 0)
                .unwrap_or(units.len());
            let result = String::from_utf16(&units[..count]).map_err(|_| ProductError::Clipboard);
            let _ = GlobalUnlock(memory);
            result.map(Some)
        }
    }
    pub fn begin_text(&self, text: &str, restore: bool) -> Result<ClipboardLease, ProductError> {
        self.begin_formats(&[(13, text_bytes(text))], restore)
    }
    pub fn begin_files(
        &self,
        paths: &[std::path::PathBuf],
        restore: bool,
    ) -> Result<ClipboardLease, ProductError> {
        self.begin_formats(&file_formats(paths)?, restore)
    }
    pub fn replace_text(&self, lease: &mut ClipboardLease, text: &str) -> Result<(), ProductError> {
        self.replace_formats(lease, &[(13, text_bytes(text))])
    }
    pub fn replace_files(
        &self,
        lease: &mut ClipboardLease,
        paths: &[std::path::PathBuf],
    ) -> Result<(), ProductError> {
        self.replace_formats(lease, &file_formats(paths)?)
    }
    fn replace_formats(
        &self,
        lease: &mut ClipboardLease,
        formats: &[(u32, Vec<u8>)],
    ) -> Result<(), ProductError> {
        let _locked = self.lock()?;
        unsafe {
            if lease.sequence == 0 || GetClipboardSequenceNumber() != lease.sequence {
                return Err(ProductError::Clipboard);
            }
            let result = write_formats(formats);
            lease.sequence = GetClipboardSequenceNumber();
            result
        }
    }
    fn begin_formats(
        &self,
        formats: &[(u32, Vec<u8>)],
        restore: bool,
    ) -> Result<ClipboardLease, ProductError> {
        let _locked = self.lock()?;
        let snapshot = if restore { Some(capture()?) } else { None };
        unsafe {
            if write_formats(formats).is_err() {
                if let Some(snapshot) = &snapshot {
                    let _ = restore_snapshot(snapshot);
                }
                return Err(ProductError::Clipboard);
            }
            let incomplete = snapshot.as_ref().is_some_and(|snapshot| !snapshot.complete);
            Ok(ClipboardLease {
                snapshot,
                sequence: GetClipboardSequenceNumber(),
                incomplete,
            })
        }
    }
    pub fn finish(&self, lease: ClipboardLease) -> Result<bool, ProductError> {
        let Some(snapshot) = lease.snapshot else {
            return Ok(false);
        };
        let _locked = self.lock()?;
        // SAFETY: sequence comparison and restoration occur under the same lock;
        // no newer clipboard writer can be overwritten between them.
        unsafe {
            if lease.sequence == 0 || GetClipboardSequenceNumber() != lease.sequence {
                return Ok(false);
            }
            restore_snapshot(&snapshot)
        }
    }
}
fn text_bytes(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}
fn file_formats(paths: &[std::path::PathBuf]) -> Result<Vec<(u32, Vec<u8>)>, ProductError> {
    let copy_format = unsafe { RegisterClipboardFormatW(w!("Preferred DropEffect")) };
    if copy_format == 0 {
        return Err(ProductError::Clipboard);
    }
    Ok(vec![
        (15, file_bytes(paths)?),
        (copy_format, 1u32.to_le_bytes().to_vec()),
    ])
}
// The caller holds the clipboard lock. All allocations precede EmptyClipboard.
fn write_formats(formats: &[(u32, Vec<u8>)]) -> Result<(), ProductError> {
    let mut prepared = Vec::new();
    unsafe {
        for (format, bytes) in formats {
            match allocate(bytes) {
                Ok(memory) => prepared.push((*format, memory)),
                Err(error) => {
                    for (_, memory) in prepared {
                        let _ = GlobalFree(Some(memory));
                    }
                    return Err(error);
                }
            }
        }
        if EmptyClipboard().is_err() {
            for (_, memory) in prepared {
                let _ = GlobalFree(Some(memory));
            }
            return Err(ProductError::Clipboard);
        }
        let mut prepared = prepared.into_iter();
        while let Some((format, memory)) = prepared.next() {
            if SetClipboardData(format, Some(HANDLE(memory.0))).is_err() {
                let _ = GlobalFree(Some(memory));
                for (_, memory) in prepared {
                    let _ = GlobalFree(Some(memory));
                }
                return Err(ProductError::Clipboard);
            }
        }
    }
    Ok(())
}
impl Drop for PasteBoard {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.owner);
        }
    }
}
pub struct ClipboardLease {
    snapshot: Option<Snapshot>,
    sequence: u32,
    pub incomplete: bool,
}
struct Snapshot {
    entries: Vec<(u32, Vec<u8>)>,
    complete: bool,
}
// Caller must hold the clipboard lock; allocate every format before emptying it.
fn restore_snapshot(snapshot: &Snapshot) -> Result<bool, ProductError> {
    unsafe {
        let mut prepared = Vec::new();
        for (format, bytes) in &snapshot.entries {
            match allocate(bytes) {
                Ok(memory) => prepared.push((*format, memory)),
                Err(error) => {
                    for (_, memory) in prepared {
                        let _ = GlobalFree(Some(memory));
                    }
                    return Err(error);
                }
            }
        }
        if EmptyClipboard().is_err() {
            for (_, memory) in prepared {
                let _ = GlobalFree(Some(memory));
            }
            return Err(ProductError::Clipboard);
        }
        let mut complete = snapshot.complete;
        for (format, memory) in prepared {
            if SetClipboardData(format, Some(HANDLE(memory.0))).is_err() {
                let _ = GlobalFree(Some(memory));
                complete = false;
            }
        }
        Ok(complete)
    }
}
fn capture() -> Result<Snapshot, ProductError> {
    let mut entries = Vec::new();
    let mut complete = true;
    let mut total = 0usize;
    let mut format = 0;
    // SAFETY: only called while clipboard is locked. Never treat GDI handles as memory.
    unsafe {
        loop {
            format = EnumClipboardFormats(format);
            if format == 0 {
                break;
            }
            if entries.len() >= 128 {
                return Err(ProductError::Clipboard);
            }
            if matches!(
                format,
                2 | 3 | 9 | 14 | 0x80 | 0x82 | 0x83 | 0x8E | 0x200..=0x3FF
            ) {
                complete = false;
                continue;
            }
            let Ok(handle) = GetClipboardData(format) else {
                complete = false;
                continue;
            };
            let memory = HGLOBAL(handle.0);
            if GlobalFlags(memory) == 0x8000 {
                complete = false;
                continue;
            }
            let size = GlobalSize(memory);
            if size == 0 || size > 16 * 1024 * 1024 || total.saturating_add(size) > 48 * 1024 * 1024
            {
                complete = false;
                continue;
            }
            let pointer = GlobalLock(memory);
            if pointer.is_null() {
                complete = false;
                continue;
            }
            let bytes = std::slice::from_raw_parts(pointer.cast::<u8>(), size).to_vec();
            let _ = GlobalUnlock(memory);
            total += size;
            entries.push((format, bytes));
        }
    }
    Ok(Snapshot { entries, complete })
}
fn allocate(bytes: &[u8]) -> Result<HGLOBAL, ProductError> {
    // SAFETY: owned allocation is either transferred to Windows or freed on error.
    unsafe {
        let memory =
            GlobalAlloc(GMEM_MOVEABLE, bytes.len()).map_err(|_| ProductError::Clipboard)?;
        let pointer = GlobalLock(memory);
        if pointer.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err(ProductError::Clipboard);
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer.cast::<u8>(), bytes.len());
        let _ = GlobalUnlock(memory);
        Ok(memory)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_payload_has_wide_drop_header_and_keeps_exact_path_order() {
        let paths = vec![
            std::path::PathBuf::from("C:\\owned\\first.txt"),
            std::path::PathBuf::from("C:\\owned\\résumé.txt"),
        ];
        let bytes = file_bytes(&paths).unwrap();
        assert_eq!(u32::from_le_bytes(bytes[..4].try_into().unwrap()), 20);
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 1);
        let units: Vec<u16> = bytes[20..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        assert_eq!(
            String::from_utf16(&units).unwrap(),
            "C:\\owned\\first.txt\0C:\\owned\\résumé.txt\0\0"
        );
        assert!(file_bytes(&[std::path::PathBuf::from("relative.txt")]).is_err());
    }
}
