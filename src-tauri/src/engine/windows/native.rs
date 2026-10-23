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
            Input::{
                Ime::{
                    GCS_COMPSTR, ImmGetCompositionStringW, ImmGetContext, ImmIsIME,
                    ImmReleaseContext,
                },
                KeyboardAndMouse::{
                    GetAsyncKeyState, GetKeyboardLayout, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN,
                    VK_SHIFT,
                },
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
        })
    }
}

pub(super) fn check_input_state(stamp: &TargetStamp) -> Result<(), RefusalReason> {
    if current_stamp()? != *stamp {
        return Err(RefusalReason::StaleContext);
    }
    // SAFETY: read-only keyboard/IMM queries on the observed thread and HWND.
    unsafe {
        let layout = GetKeyboardLayout(stamp.thread_id);
        if layout.0 as usize != 0x04090409 || ImmIsIME(layout).as_bool() {
            return Err(RefusalReason::UnsupportedLayout);
        }
        if [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
            .iter()
            .any(|key| GetAsyncKeyState(i32::from(key.0)) < 0)
        {
            return Err(RefusalReason::Modifier);
        }
        let window = HWND(stamp.focused_window as *mut _);
        let context = ImmGetContext(window);
        if !context.0.is_null() {
            let bytes = ImmGetCompositionStringW(context, GCS_COMPSTR, None, 0);
            let released = ImmReleaseContext(window, context).as_bool();
            // -1 means no composition data; other errors remain uncertain.
            if !released || !matches!(bytes, 0 | -1) {
                return Err(RefusalReason::Composition);
            }
        }
    }
    Ok(())
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
