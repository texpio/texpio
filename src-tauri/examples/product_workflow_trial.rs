//! Explicit owned-window acceptance. Never installs hooks or reads a foreign control.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
#[cfg(target_os = "windows")]
mod trial {
    use std::{
        cell::{Cell, RefCell},
        sync::{
            Arc,
            atomic::{AtomicU64, Ordering},
            mpsc,
        },
        time::Duration,
    };
    use texpio_lib::{
        engine::windows::current_stamp,
        library::LibraryStore,
        product::{
            matcher::{Expansion, ProductRule},
            payload::FileDelivery,
            settings::ProductSettings,
            windows::NativeEdit,
        },
    };
    use windows::{
        Win32::{
            Foundation::*,
            System::{DataExchange::*, LibraryLoader::GetModuleHandleW},
            UI::{
                Input::KeyboardAndMouse::*,
                Shell::{DragQueryFileW, HDROP},
                WindowsAndMessaging::*,
            },
        },
        core::w,
    };
    fn expected_text() -> &'static str {
        if CORRECTION.with(Cell::get) {
            "prefix Texpio Windows paste works."
        } else {
            "Texpio Windows paste works. "
        }
    }
    thread_local! {
        static GROUPS:RefCell<Vec<Vec<String>>>=const{RefCell::new(Vec::new())};
        static EDITOR:Cell<isize>=const{Cell::new(0)};
        static PHYSICAL:Cell<bool>=const{Cell::new(false)};
        static CORRECTION:Cell<bool>=const{Cell::new(false)};
        static TICKS:Cell<u32>=const{Cell::new(0)};
        static MATCHED_AT:Cell<Option<u32>>=const{Cell::new(None)};
    }
    unsafe extern "system" fn procedure(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_CLOSE || message == WM_DESTROY || message == WM_APP + 1 {
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
        if message == WM_TIMER {
            let expired = TICKS.with(|ticks| {
                let next = ticks.get() + 1;
                ticks.set(next);
                next >= if PHYSICAL.with(|mode| mode.get()) {
                    1800
                } else {
                    50
                }
            });
            let exact = PHYSICAL.with(|mode| mode.get())
                && EDITOR.with(|editor| read(HWND(editor.get() as *mut _)) == expected_text());
            let settled = exact
                && MATCHED_AT.with(|matched| {
                    let tick = TICKS.with(Cell::get);
                    let first = matched.get().unwrap_or(tick);
                    matched.set(Some(first));
                    tick >= first + 10
                });
            if expired || settled {
                unsafe { PostQuitMessage(0) };
            }
            return LRESULT(0);
        }
        if message == WM_KEYDOWN && wparam.0 == VK_ESCAPE.0 as usize {
            unsafe { PostQuitMessage(0) };
            return LRESULT(0);
        }
        if message == WM_KEYDOWN
            && wparam.0 == VK_V.0 as usize
            && unsafe { GetKeyState(i32::from(VK_CONTROL.0)) < 0 }
        {
            if unsafe { OpenClipboard(Some(window)) }.is_ok() {
                if let Ok(handle) = unsafe { GetClipboardData(15) } {
                    let drop = HDROP(handle.0);
                    let count = unsafe { DragQueryFileW(drop, u32::MAX, None) };
                    if count <= 100 {
                        let mut paths = Vec::new();
                        for index in 0..count {
                            let length = unsafe { DragQueryFileW(drop, index, None) };
                            if length > 32767 {
                                break;
                            }
                            let mut value = vec![0; length as usize + 1];
                            let actual = unsafe { DragQueryFileW(drop, index, Some(&mut value)) };
                            if actual != length {
                                break;
                            }
                            paths.push(String::from_utf16_lossy(&value[..length as usize]));
                        }
                        GROUPS.with(|groups| {
                            let mut groups = groups.borrow_mut();
                            if groups.len() < 100 {
                                groups.push(paths);
                            }
                        });
                    }
                }
                unsafe {
                    let _ = CloseClipboard();
                }
            }
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(window, message, wparam, lparam) }
    }
    fn read(window: HWND) -> String {
        let mut text = [0u16; 1024];
        let length = unsafe { GetWindowTextW(window, &mut text) }.max(0) as usize;
        String::from_utf16_lossy(&text[..length])
    }
    struct Owned {
        window: HWND,
        instance: HINSTANCE,
    }
    impl Drop for Owned {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyWindow(self.window);
                let _ = UnregisterClassW(w!("TexpioProductWorkflowReceiver"), Some(self.instance));
            }
        }
    }
    pub fn run(physical: bool, correction: bool) -> windows::core::Result<bool> {
        unsafe {
            PHYSICAL.with(|mode| mode.set(physical));
            CORRECTION.with(|mode| mode.set(correction));
            let instance: HINSTANCE = GetModuleHandleW(None)?.into();
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: instance,
                lpszClassName: w!("TexpioProductWorkflowReceiver"),
                ..Default::default()
            };
            if RegisterClassW(&class) == 0 {
                return Err(windows::core::Error::from_thread());
            }
            let title = if correction {
                w!("Texpio immediate correction check — no Space after correction; Escape closes")
            } else if physical {
                w!("Texpio disposable typing check — type :v1probe then Space; Escape closes")
            } else {
                w!("Texpio owned file-paste check — Escape cancels")
            };
            let window = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("TexpioProductWorkflowReceiver"),
                title,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                180,
                180,
                850,
                220,
                None,
                None,
                Some(instance),
                None,
            )?;
            let owned = Owned { window, instance };
            if physical {
                SetWindowPos(
                    window,
                    Some(HWND_TOPMOST),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                )?;
            }
            let label = if correction {
                w!(
                    "Type: prefix ;helllo — then Backspace twice, then o. No final Space. 3 minutes; Escape closes."
                )
            } else if physical {
                w!("Type :v1probe then Space once. You have 3 minutes. Escape closes this window.")
            } else {
                w!(
                    "Checking two owned files through Windows clipboard paste. No other window is targeted."
                )
            };
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                label,
                WS_CHILD | WS_VISIBLE,
                10,
                10,
                800,
                36,
                Some(window),
                None,
                Some(instance),
                None,
            )?;
            let editor = if physical {
                CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("EDIT"),
                    w!(""),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE(ES_MULTILINE as u32),
                    10,
                    52,
                    800,
                    80,
                    Some(window),
                    None,
                    Some(instance),
                    None,
                )?
            } else {
                window
            };
            EDITOR.with(|value| value.set(editor.0 as isize));
            let _ = SetForegroundWindow(window);
            SetFocus(Some(editor))?;
            if GetForegroundWindow() != window && !physical {
                println!("REFUSED foreground was not the owned fixture; no input submitted.");
                return Ok(false);
            }
            if SetTimer(Some(window), 1, 100, None) == 0 {
                return Err(windows::core::Error::from_thread());
            }
            let generation = Arc::new(AtomicU64::new(1));
            let mut worker = None;
            let mut finished = None;
            let mut expected = Vec::new();
            let mut store_root = None;
            let mut sources = None;
            if !physical {
                let root = tempfile::tempdir().map_err(|_| windows::core::Error::empty())?;
                let source = tempfile::tempdir().map_err(|_| windows::core::Error::empty())?;
                let first = source.path().join("first.txt");
                let second = source.path().join("résumé.txt");
                std::fs::write(&first, b"first owned bytes")
                    .map_err(|_| windows::core::Error::empty())?;
                std::fs::write(&second, b"second owned bytes")
                    .map_err(|_| windows::core::Error::empty())?;
                let store = LibraryStore::new(root.path().into());
                let library = store.load().map_err(|_| windows::core::Error::empty())?;
                let id = &library.snippets[0].id;
                store
                    .add_attachments(id, &[first, second])
                    .map_err(|_| windows::core::Error::empty())?;
                let paths = store
                    .delivery_files(id)
                    .map_err(|_| windows::core::Error::empty())?;
                expected = paths
                    .iter()
                    .map(|path| vec![path.to_string_lossy().into_owned()])
                    .collect();
                let target = current_stamp().map_err(|_| windows::core::Error::empty())?;
                let cancel = generation.clone();
                let handle = window.0 as isize;
                let (sender, receiver) = mpsc::sync_channel(1);
                worker = Some(std::thread::spawn(move || {
                    let settings = ProductSettings {
                        preserve_terminator: false,
                        attachment_delay_ms: 200,
                        ..Default::default()
                    };
                    let result = NativeEdit::new(target, cancel, 1, settings).and_then(|edit| {
                        let mut edit = edit.require_complete_clipboard().with_files(FileDelivery {
                            files: paths,
                            first: true,
                            strict: true,
                        });
                        let result = edit.deliver(&Expansion {
                            rule: ProductRule {
                                id: "owned-file-fixture".into(),
                                trigger: String::new(),
                                body: "".into(),
                            },
                            erase_count: 0,
                            terminator: None,
                        });
                        result.map(|_| edit.submitted_events)
                    });
                    let _ = sender.send(result);
                    let _ = PostMessageW(
                        Some(HWND(handle as *mut _)),
                        WM_APP + 1,
                        WPARAM(0),
                        LPARAM(0),
                    );
                }));
                finished = Some(receiver);
                store_root = Some(root);
                sources = Some(source);
            } else {
                if !IsWindowVisible(window).as_bool() {
                    println!("REFUSED owned fixture was not visible; no input submitted.");
                    return Ok(false);
                }
                println!("READY visible owned physical fixture; no synthetic typing is generated.");
            }
            let mut message = MSG::default();
            while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                if message.message == WM_KEYDOWN && message.wParam.0 == VK_ESCAPE.0 as usize {
                    break;
                }
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            generation.fetch_add(1, Ordering::SeqCst);
            let passed = if physical {
                let exact = read(editor) == expected_text();
                println!(
                    "Owned typing fixture exact={exact}; window closed. No physical qualification count is inferred by this helper."
                );
                exact
            } else {
                let outcome = finished
                    .and_then(|receiver| receiver.recv_timeout(Duration::from_millis(750)).ok());
                let exact = GROUPS.with(|groups| *groups.borrow() == expected);
                let submitted = matches!(outcome, Some(Ok(8)));
                println!(
                    "Owned file paste: groups_exact={exact}, expected_groups=2, submitted_exact={submitted}; window closed. No retry."
                );
                if outcome.is_some()
                    && let Some(worker) = worker.take()
                {
                    let _ = worker.join();
                }
                exact && submitted
            };
            drop(owned);
            drop(store_root);
            drop(sources);
            Ok(passed)
        }
    }
}
#[cfg(target_os = "windows")]
fn main() -> std::process::ExitCode {
    let (physical, correction) = match std::env::args().nth(1).as_deref() {
        Some("--physical-paste") => (true, false),
        Some("--immediate-correction") => (true, true),
        Some("--file-paste") => (false, false),
        _ => {
            eprintln!(
                "Usage: product_workflow_trial --physical-paste | --immediate-correction | --file-paste"
            );
            return std::process::ExitCode::FAILURE;
        }
    };
    if matches!(trial::run(physical, correction), Ok(true)) {
        std::process::ExitCode::SUCCESS
    } else {
        println!("Owned workflow not confirmed; diagnostic ended without replay.");
        std::process::ExitCode::FAILURE
    }
}
#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("This explicit diagnostic requires Windows.");
}
