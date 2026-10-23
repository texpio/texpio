//! Small Win32 observations. No hooks, input synthesis, UIA, or target text here.
use super::super::types::RefusalReason;
use std::mem::size_of;
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND},
        Security::{
            GetSidIdentifierAuthority, GetSidSubAuthority, GetSidSubAuthorityCount,
            GetTokenInformation, IsValidSid, TOKEN_ELEVATION, TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
            TokenElevation, TokenIntegrityLevel,
        },
        System::Threading::{
            OpenProcess, OpenProcessToken, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
        UI::{
            Input::KeyboardAndMouse::{
                GetAsyncKeyState, GetKeyboardLayout, HKL, MAPVK_VK_TO_VSC, MapVirtualKeyExW,
                ToUnicodeEx, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
            },
            WindowsAndMessaging::{
                GUITHREADINFO, GetForegroundWindow, GetGUIThreadInfo, GetWindowTextW,
                GetWindowThreadProcessId,
            },
        },
    },
    core::PWSTR,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct TargetStamp {
    pub foreground_window: isize,
    pub focused_window: isize,
    pub process_id: u32,
    pub thread_id: u32,
    pub keyboard_layout: usize,
}

pub fn current_stamp() -> Result<TargetStamp, RefusalReason> {
    // SAFETY: caller-owned initialized buffers; borrowed handles are never closed.
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.is_invalid() {
            return Err(RefusalReason::ContextUnavailable);
        }
        let mut process_id = 0;
        let thread_id = GetWindowThreadProcessId(foreground, Some(&mut process_id));
        let mut info = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        GetGUIThreadInfo(thread_id, &mut info).map_err(|_| RefusalReason::ContextUnavailable)?;
        if process_id == 0
            || thread_id == 0
            || info.hwndFocus.is_invalid()
            || info.hwndActive != foreground
            || GetForegroundWindow() != foreground
        {
            return Err(RefusalReason::StaleContext);
        }
        Ok(TargetStamp {
            foreground_window: foreground.0 as isize,
            focused_window: info.hwndFocus.0 as isize,
            process_id,
            thread_id,
            keyboard_layout: GetKeyboardLayout(thread_id).0 as usize,
        })
    }
}

pub(super) fn check_input_state(stamp: &TargetStamp) -> Result<(), RefusalReason> {
    if current_stamp()? != *stamp {
        return Err(RefusalReason::StaleContext);
    }
    // SAFETY: read-only keyboard queries on the observed thread.
    unsafe {
        let layout = GetKeyboardLayout(stamp.thread_id);
        if layout.0 as usize != stamp.keyboard_layout
            || !candidate_layout(stamp.keyboard_layout)
            || !verified_mapping(layout)
        {
            return Err(RefusalReason::UnsupportedLayout);
        }
        if [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
            .iter()
            .any(|key| GetAsyncKeyState(i32::from(key.0)) < 0)
        {
            return Err(RefusalReason::Modifier);
        }
    }
    // Composition is checked through the captured control's UIA TextEdit pattern.
    // Foreign-thread IMM context queries cannot establish that another app is idle.
    Ok(())
}

fn candidate_layout(layout: usize) -> bool {
    // HKLs measured/selected for this P0; language IDs alone do not identify layouts.
    matches!(layout, 0x04090409 | 0x40094009 | 0x40090409)
}

fn mapping_matches(mut matches: impl FnMut(u32, u16) -> bool) -> bool {
    (0x41..=0x5A).all(|key| matches(key, (key - 0x41) as u16 + u16::from(b'a')))
        && matches(0xBA, u16::from(b';'))
        && matches(0x20, u16::from(b' '))
}

fn verified_mapping(layout: HKL) -> bool {
    mapping_matches(|key, expected| {
        let mut output = [0u16; 4];
        // SAFETY: read-only translation with no held modifiers/Caps Lock. Flag 4
        // preserves the kernel keyboard state (Windows 10 1607+). No keys are sent.
        // MapVirtualKey(VK_TO_CHAR) cannot prove letter mappings: it forces A-Z.
        unsafe {
            let scan = MapVirtualKeyExW(key, MAPVK_VK_TO_VSC, Some(layout));
            scan != 0
                && ToUnicodeEx(key, scan, &[0; 256], &mut output, 4, Some(layout)) == 1
                && output[0] == expected
        }
    })
}

pub(super) struct ProcessIdentity {
    pub handle: OwnedHandle,
    pub executable: String,
}
pub(super) struct OwnedHandle(pub HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: only successful owned process/token handles enter this wrapper.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn standard_medium_process(process: HANDLE) -> Result<(), RefusalReason> {
    // SAFETY: token buffers remain alive and aligned while Windows/SID APIs use them.
    unsafe {
        let mut raw_token = HANDLE::default();
        OpenProcessToken(process, TOKEN_QUERY, &mut raw_token)
            .map_err(|_| RefusalReason::Integrity)?;
        let token = OwnedHandle(raw_token);
        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0;
        GetTokenInformation(
            token.0,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
        .map_err(|_| RefusalReason::Integrity)?;
        if elevation.TokenIsElevated != 0 || returned as usize != size_of::<TOKEN_ELEVATION>() {
            return Err(RefusalReason::Integrity);
        }
        let mut bytes = 0;
        let _ = GetTokenInformation(token.0, TokenIntegrityLevel, None, 0, &mut bytes);
        if !(size_of::<TOKEN_MANDATORY_LABEL>()..=4096).contains(&(bytes as usize)) {
            return Err(RefusalReason::Integrity);
        }
        let mut buffer = vec![0usize; (bytes as usize).div_ceil(size_of::<usize>())];
        GetTokenInformation(
            token.0,
            TokenIntegrityLevel,
            Some(buffer.as_mut_ptr().cast()),
            bytes,
            &mut returned,
        )
        .map_err(|_| RefusalReason::Integrity)?;
        if returned > bytes {
            return Err(RefusalReason::Integrity);
        }
        let label = &*buffer.as_ptr().cast::<TOKEN_MANDATORY_LABEL>();
        let sid = label.Label.Sid;
        if !IsValidSid(sid).as_bool()
            || *GetSidSubAuthorityCount(sid) != 1
            || (*GetSidIdentifierAuthority(sid)).Value != [0, 0, 0, 0, 0, 16]
            || *GetSidSubAuthority(sid, 0) != 0x2000
        {
            return Err(RefusalReason::Integrity);
        }
    }
    Ok(())
}

pub(super) fn process_identity(process_id: u32) -> Result<ProcessIdentity, RefusalReason> {
    // SAFETY: query-only handle is owned and closed through RAII.
    unsafe {
        let process = OwnedHandle(
            OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id)
                .map_err(|_| RefusalReason::Integrity)?,
        );
        standard_medium_process(process.0)?;
        // Compare with the actual current process, never close its pseudo handle.
        standard_medium_process(windows::Win32::System::Threading::GetCurrentProcess())?;
        let mut path = vec![0u16; 32768];
        let mut length = path.len() as u32;
        QueryFullProcessImageNameW(
            process.0,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut length,
        )
        .map_err(|_| RefusalReason::UnsupportedProfile)?;
        let executable = String::from_utf16(&path[..length as usize])
            .map_err(|_| RefusalReason::UnsupportedProfile)?;
        Ok(ProcessIdentity {
            handle: process,
            executable: executable.rsplit('\\').next().unwrap_or("").into(),
        })
    }
}

pub(super) fn caption(stamp: &TargetStamp) -> Result<String, RefusalReason> {
    let mut buffer = [0u16; 512];
    // SAFETY: bounded caller-owned output buffer; target HWND is only queried.
    let count = unsafe { GetWindowTextW(HWND(stamp.foreground_window as *mut _), &mut buffer) };
    if count <= 0 || count as usize >= buffer.len() - 1 {
        return Err(RefusalReason::UnsupportedProfile);
    }
    String::from_utf16(&buffer[..count as usize]).map_err(|_| RefusalReason::UnsupportedProfile)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_us_and_explicit_india_layout_handles_are_candidates() {
        for layout in [0x04090409, 0x40094009, 0x40090409] {
            assert!(candidate_layout(layout));
        }
        for layout in [0, 0x08090809, 0x00010409, 0xE0010409, 0x40090809] {
            assert!(!candidate_layout(layout));
        }
    }

    #[test]
    fn recognition_mapping_requires_each_exact_single_character() {
        assert!(mapping_matches(|key, expected| key != 0 && expected != 0));
        assert!(!mapping_matches(|key, _| key != 0xBA));
        assert!(!mapping_matches(|key, _| key != 0x51));
        assert!(!mapping_matches(|key, _| key != 0x20));
    }
}
