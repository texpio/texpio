//! Windows effects. Explicitly invoked; constructing this module installs nothing.
use super::{
    clipboard::{ClipboardLease, PasteBoard},
    effects::{self, EffectBackend, ProductError},
    matcher::{Expansion, ProductRule},
    settings::{InsertionMode, ProductSettings},
};
use crate::engine::windows::{TargetStamp, current_stamp};
use std::{
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::{Duration, Instant},
};
use windows::Win32::UI::{
    Input::KeyboardAndMouse::*,
    WindowsAndMessaging::{ES_PASSWORD, GWL_STYLE, GetWindowLongPtrW},
};

pub const PRODUCT_MARKER: usize = 0x54585056;

fn event(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: PRODUCT_MARKER,
            },
        },
    }
}
fn press(key: VIRTUAL_KEY) -> Vec<INPUT> {
    vec![
        event(key, 0, KEYBD_EVENT_FLAGS(0)),
        event(key, 0, KEYEVENTF_KEYUP),
    ]
}
fn character(ch: char) -> Vec<INPUT> {
    if ch == '\n' {
        return press(VK_RETURN);
    }
    let mut buffer = [0u16; 2];
    let mut events = Vec::with_capacity(4);
    for unit in ch.encode_utf16(&mut buffer) {
        events.push(event(VIRTUAL_KEY(0), *unit, KEYEVENTF_UNICODE));
        events.push(event(
            VIRTUAL_KEY(0),
            *unit,
            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        ));
    }
    events
}

pub struct NativeEdit {
    target: TargetStamp,
    generation: Arc<AtomicU64>,
    ticket: u64,
    settings: ProductSettings,
    armed_cancel: Cell<bool>,
    next_send: Instant,
    minimum_interval: Duration,
    board: Option<PasteBoard>,
    lease: Option<ClipboardLease>,
    files: super::payload::FileDelivery,
    pub submitted_events: u32,
    pub clipboard_incomplete: bool,
}

impl NativeEdit {
    pub fn new(
        target: TargetStamp,
        generation: Arc<AtomicU64>,
        ticket: u64,
        settings: ProductSettings,
    ) -> Result<Self, ProductError> {
        if !settings.validate() {
            return Err(ProductError::InvalidSettings);
        }
        let cancel = settings.cancel_vk().ok_or(ProductError::InvalidSettings)?;
        // SAFETY: read-only modifier/cancel and control style observations.
        let armed_cancel = unsafe { GetAsyncKeyState(i32::from(cancel)) >= 0 };
        if unsafe {
            GetWindowLongPtrW(
                windows::Win32::Foundation::HWND(target.focused_window as *mut _),
                GWL_STYLE,
            )
        } as u32
            & ES_PASSWORD as u32
            != 0
        {
            return Err(ProductError::Unavailable);
        }
        Ok(Self {
            target,
            generation,
            ticket,
            settings,
            armed_cancel: Cell::new(armed_cancel),
            next_send: Instant::now(),
            minimum_interval: Duration::ZERO,
            board: None,
            lease: None,
            files: Default::default(),
            submitted_events: 0,
            clipboard_incomplete: false,
        })
    }
    pub fn with_files(mut self, files: super::payload::FileDelivery) -> Self {
        self.files = files;
        self
    }
    fn paste_shortcut(&mut self) -> Result<(), ProductError> {
        self.send(
            &[
                event(VK_CONTROL, 0, KEYBD_EVENT_FLAGS(0)),
                event(VK_V, 0, KEYBD_EVENT_FLAGS(0)),
                event(VK_V, 0, KEYEVENTF_KEYUP),
                event(VK_CONTROL, 0, KEYEVENTF_KEYUP),
            ],
            false,
        )
    }
    fn paste_files(&mut self) -> Result<(), ProductError> {
        let groups = if self.files.strict {
            self.files
                .files
                .iter()
                .cloned()
                .map(|file| vec![file])
                .collect::<Vec<_>>()
        } else {
            vec![self.files.files.clone()]
        };
        for files in groups {
            self.check()?;
            let board = self.board.as_ref().ok_or(ProductError::Clipboard)?;
            let lease = self.lease.as_mut().ok_or(ProductError::Clipboard)?;
            board.replace_files(lease, &files)?;
            self.paste_shortcut()?;
            self.settle(Duration::from_millis(u64::from(
                self.settings.attachment_delay_ms,
            )))?;
        }
        Ok(())
    }
    fn send(&mut self, events: &[INPUT], pace: bool) -> Result<(), ProductError> {
        self.check()?;
        if pace {
            while self.next_send > Instant::now() {
                self.check()?;
                thread::sleep(
                    self.next_send
                        .saturating_duration_since(Instant::now())
                        .min(Duration::from_millis(4)),
                );
            }
        }
        self.check()?;
        // SAFETY: bounded initialized pairs; no pointers escape this call.
        let sent = unsafe { SendInput(events, std::mem::size_of::<INPUT>() as i32) };
        self.submitted_events = self.submitted_events.saturating_add(sent);
        self.next_send = Instant::now()
            + Duration::from_secs_f64(
                events.len() as f64 / f64::from(self.settings.typing_speed.events_per_second()),
            )
            .max(self.minimum_interval);
        if sent as usize != events.len() {
            return Err(ProductError::InputBlocked);
        }
        Ok(())
    }
    fn settle(&mut self, duration: Duration) -> Result<(), ProductError> {
        let deadline = Instant::now() + duration;
        while Instant::now() < deadline {
            self.check()?;
            thread::sleep(Duration::from_millis(4));
        }
        self.check()
    }
    fn finish_clipboard(&mut self) -> Result<(), ProductError> {
        if let (Some(board), Some(lease)) = (self.board.as_ref(), self.lease.take()) {
            self.clipboard_incomplete |= lease.incomplete;
            let _ = board.finish(lease)?;
        }
        Ok(())
    }
    pub fn deliver(&mut self, expansion: &Expansion) -> Result<(), ProductError> {
        let settings = self.settings.clone();
        let result = effects::perform(self, expansion, &settings);
        let restored = self.finish_clipboard();
        result.and(restored)
    }
}
impl Drop for NativeEdit {
    fn drop(&mut self) {
        let _ = self.finish_clipboard();
    }
}
impl EffectBackend for NativeEdit {
    fn check(&mut self) -> Result<(), ProductError> {
        if self.generation.load(Ordering::SeqCst) != self.ticket {
            return Err(ProductError::Cancelled);
        }
        if current_stamp().ok() != Some(self.target) {
            return Err(ProductError::TargetChanged);
        }
        // SAFETY: read-only instantaneous key state. Held modifiers are never forced up.
        unsafe {
            let down = GetAsyncKeyState(i32::from(
                self.settings
                    .cancel_vk()
                    .ok_or(ProductError::InvalidSettings)?,
            )) < 0;
            if !down {
                self.armed_cancel.set(true);
            } else if self.armed_cancel.get() {
                return Err(ProductError::Cancelled);
            }
            if [VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN]
                .iter()
                .any(|key| GetAsyncKeyState(i32::from(key.0)) < 0)
            {
                return Err(ProductError::ModifiersHeld);
            }
        }
        Ok(())
    }
    fn prepare(&mut self, text: &str, mode: InsertionMode) -> Result<(), ProductError> {
        if !self.files.files.is_empty() || (mode == InsertionMode::Paste && !text.is_empty()) {
            let board = PasteBoard::new()?;
            let lease = if !self.files.files.is_empty()
                && (self.files.first || text.is_empty() || mode == InsertionMode::Type)
            {
                board.begin_files(
                    if self.files.strict {
                        &self.files.files[..1]
                    } else {
                        &self.files.files
                    },
                    self.settings.restore_clipboard,
                )?
            } else {
                board.begin_text(text, self.settings.restore_clipboard)?
            };
            self.clipboard_incomplete = lease.incomplete;
            self.board = Some(board);
            self.lease = Some(lease);
        }
        Ok(())
    }
    fn erase(&mut self, characters: usize) -> Result<(), ProductError> {
        for _ in 0..characters {
            self.send(&press(VK_BACK), true)?;
        }
        Ok(())
    }
    fn content(&mut self, text: &str, mode: InsertionMode) -> Result<(), ProductError> {
        match mode {
            InsertionMode::Type => {
                // CRLF is one Return; standalone CR is also a Return.
                let mut characters = text.chars().peekable();
                while let Some(ch) = characters.next() {
                    let ch = if ch == '\r' {
                        if characters.peek() == Some(&'\n') {
                            characters.next();
                        }
                        '\n'
                    } else {
                        ch
                    };
                    self.send(&character(ch), true)?;
                }
                Ok(())
            }
            InsertionMode::Paste => {
                if !self.files.files.is_empty() {
                    self.board
                        .as_ref()
                        .ok_or(ProductError::Clipboard)?
                        .replace_text(self.lease.as_mut().ok_or(ProductError::Clipboard)?, text)?;
                }
                self.settle(Duration::from_millis(25))?;
                self.send(
                    &[
                        event(VK_CONTROL, 0, KEYBD_EVENT_FLAGS(0)),
                        event(VK_V, 0, KEYBD_EVENT_FLAGS(0)),
                        event(VK_V, 0, KEYEVENTF_KEYUP),
                        event(VK_CONTROL, 0, KEYEVENTF_KEYUP),
                    ],
                    false,
                )?;
                self.settle(Duration::from_millis(200))
            }
        }
    }
    fn terminator(&mut self, ch: char) -> Result<(), ProductError> {
        self.send(
            &match ch {
                '\t' => press(VK_TAB),
                '\r' | '\n' => press(VK_RETURN),
                _ => character(ch),
            },
            true,
        )
    }
    fn before_content(&mut self) -> Result<(), ProductError> {
        if self.files.first && !self.files.files.is_empty() {
            self.paste_files()?;
        }
        Ok(())
    }
    fn after_content(&mut self) -> Result<(), ProductError> {
        if !self.files.first && !self.files.files.is_empty() {
            self.paste_files()?;
        }
        Ok(())
    }
}

/// Harness-only entry: caller has independently validated its fixed fixture.
/// It deletes an already-delivered terminator too, unlike the hook's suppression path.
pub fn diagnostic_dispatch(
    target: TargetStamp,
    delete_count: usize,
    output: &str,
    mode: InsertionMode,
) -> Result<(u32, u32), ProductError> {
    if delete_count > 129 || output.chars().count() > 1_000_000 {
        return Err(ProductError::Unavailable);
    }
    let generation = Arc::new(AtomicU64::new(1));
    let settings = ProductSettings {
        insertion: mode,
        preserve_terminator: false,
        ..Default::default()
    };
    let mut native = NativeEdit::new(target, generation, 1, settings)?;
    // Debug fixture experiment only; never changes normal product settings.
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("TEXPIO_TRIAL_INTERVAL_MS") {
        let milliseconds = value
            .parse::<u64>()
            .ok()
            .filter(|value| *value <= 100)
            .ok_or(ProductError::InvalidSettings)?;
        native.minimum_interval = Duration::from_millis(milliseconds);
        println!("Diagnostic minimum interval: {milliseconds} ms");
    }
    let expansion = Expansion {
        rule: ProductRule {
            id: "fixture".into(),
            trigger: "x".repeat(delete_count),
            body: output.into(),
        },
        terminator: ' ',
    };
    native.deliver(&expansion)?;
    let expected = delete_count as u32 * 2
        + match mode {
            InsertionMode::Paste if output.is_empty() => 0,
            InsertionMode::Paste => 4,
            InsertionMode::Type => (output.replace("\r\n", "\n").encode_utf16().count() * 2) as u32,
        };
    Ok((expected, native.submitted_events))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoding_keeps_supplementary_scalar_pairs_together_and_tabs_literal() {
        let emoji = character('👋');
        assert_eq!(emoji.len(), 4);
        let units: Vec<_> = emoji
            .iter()
            .step_by(2)
            .map(|input| unsafe { input.Anonymous.ki.wScan })
            .collect();
        assert_eq!(units, "👋".encode_utf16().collect::<Vec<_>>());
        assert_eq!(unsafe { character('\t')[0].Anonymous.ki.wScan }, 9);
        assert_eq!(unsafe { character('\n')[0].Anonymous.ki.wVk }, VK_RETURN);
        assert!(
            emoji
                .iter()
                .all(|input| unsafe { input.Anonymous.ki.dwExtraInfo == PRODUCT_MARKER })
        );
    }
}
