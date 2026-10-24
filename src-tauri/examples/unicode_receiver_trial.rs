//! Explicit owned-window diagnostic. Never installs hooks or touches another app.
#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    use std::mem::size_of;
    use windows::{
        Win32::{
            Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::{
                Input::KeyboardAndMouse::{
                    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
                    KEYEVENTF_UNICODE, SendInput, SetFocus, VIRTUAL_KEY, VK_BACK, VK_CONTROL,
                    VK_ESCAPE, VK_LWIN, VK_MENU, VK_PACKET, VK_RWIN, VK_SHIFT,
                },
                WindowsAndMessaging::*,
            },
        },
        core::w,
    };

    if std::env::args().nth(1).as_deref() != Some("--owned-receiver") {
        eprintln!("Usage: unicode_receiver_trial --owned-receiver");
        return std::process::ExitCode::FAILURE;
    }
    unsafe extern "system" fn procedure(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_TIMER || message == WM_DESTROY {
            // SAFETY: this diagnostic owns the current thread's message loop.
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
        // SAFETY: forwarding the unchanged message for this owned window.
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
    struct OwnedWindow {
        window: HWND,
        instance: HINSTANCE,
    }
    impl Drop for OwnedWindow {
        fn drop(&mut self) {
            // SAFETY: exact resources created on this thread; children/timer die
            // with their parent. No foreign window, hook or process is closed.
            unsafe {
                let _ = DestroyWindow(self.window);
                let _ = UnregisterClassW(w!("TexpioUnicodeReceiver"), Some(self.instance));
            }
        }
    }
    fn key(vk: VIRTUAL_KEY, scan: u16, up: bool, unicode: bool) -> INPUT {
        let mut flags = if unicode {
            KEYEVENTF_UNICODE
        } else {
            Default::default()
        };
        if up {
            flags |= KEYEVENTF_KEYUP;
        }
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: scan,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: texpio_lib::engine::windows::unicode::OWN_INPUT_MARKER,
                },
            },
        }
    }
    let result = (|| -> windows::core::Result<bool> {
        // SAFETY: this helper creates and owns both HWNDs. No provider or target
        // outside this process is queried or mutated.
        unsafe {
            let instance: HINSTANCE = GetModuleHandleW(None)?.into();
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance,
                lpszClassName: w!("TexpioUnicodeReceiver"),
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(windows::core::Error::from_thread());
            }
            let window = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("TexpioUnicodeReceiver"),
                w!("Texpio owned Unicode diagnostic — Escape stops"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                120,
                120,
                700,
                180,
                None,
                None,
                Some(instance),
                None,
            )?;
            let owned = OwnedWindow { window, instance };
            let editor = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("EDIT"),
                w!(";hello "),
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(ES_MULTILINE as u32),
                10,
                10,
                650,
                80,
                Some(owned.window),
                None,
                Some(instance),
                None,
            )?;
            let _ = SetForegroundWindow(owned.window);
            SetFocus(Some(editor))?;
            // EM_SETSEL: collapse at the end of the exact owned initial string.
            SendMessageW(editor, 0x00B1, Some(WPARAM(7)), Some(LPARAM(7)));
            let selection = SendMessageW(editor, 0x00B0, None, None).0 as u32;
            if selection & 0xFFFF != 7 || selection >> 16 != 7 {
                return Err(windows::core::Error::empty());
            }
            if GetForegroundWindow() != owned.window
                || [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
                    .iter()
                    .any(|key| GetAsyncKeyState(i32::from(key.0)) < 0)
            {
                return Err(windows::core::Error::empty());
            }
            let output = "Hello from Texpio! ";
            let mut events = Vec::new();
            for _ in 0..7 {
                events.push(key(VK_BACK, 0, false, false));
                events.push(key(VK_BACK, 0, true, false));
            }
            for unit in output.encode_utf16() {
                events.push(key(VIRTUAL_KEY(0), unit, false, true));
                events.push(key(VIRTUAL_KEY(0), unit, true, true));
            }
            if SetTimer(Some(owned.window), 1, 1800, None) == 0 {
                return Err(windows::core::Error::from_thread());
            }
            let submitted = SendInput(&events, size_of::<INPUT>() as i32);
            println!(
                "Owned receiver submission: expected={}, submitted={submitted}",
                events.len()
            );
            let mut message = MSG::default();
            let mut packets = 0;
            let mut cancelled = false;
            let mut chars = Vec::with_capacity(64);
            while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                if message.message == WM_KEYDOWN && message.wParam.0 == VK_ESCAPE.0 as usize {
                    cancelled = true;
                    break;
                }
                if message.hwnd == editor {
                    if message.message == WM_KEYDOWN && message.wParam.0 == VK_PACKET.0 as usize {
                        packets += 1;
                    }
                    if message.message == WM_CHAR && message.wParam.0 != VK_BACK.0 as usize {
                        if chars.len() == 64 {
                            cancelled = true;
                            break;
                        }
                        chars.push(message.wParam.0 as u16);
                    }
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            let mut text = [0u16; 64];
            let length = GetWindowTextW(editor, &mut text).max(0) as usize;
            let expected: Vec<u16> = output.encode_utf16().collect();
            let first_difference = chars.iter().zip(&expected).position(|(a, b)| a != b);
            println!(
                "Owned receiver: packets={packets}, chars={}, chars_exact={}, control_units={length}, control_exact={}, first_char_difference={first_difference:?}, cancelled={cancelled}",
                chars.len(),
                chars == expected,
                text[..length] == expected
            );
            Ok(!cancelled
                && submitted as usize == events.len()
                && chars == expected
                && text[..length] == expected)
        }
    })();
    if matches!(result, Ok(true)) {
        println!("OBSERVED exact owned Win32 EDIT output; diagnostic window closed.");
        std::process::ExitCode::SUCCESS
    } else {
        println!(
            "Owned receiver did not confirm exact output; diagnostic window closed. No retry."
        );
        std::process::ExitCode::FAILURE
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This explicit diagnostic requires Windows.");
}
