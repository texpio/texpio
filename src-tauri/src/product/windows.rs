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
    WindowsAndMessaging::{ES_PASSWORD, GWL_STYLE, GetClassNameW, GetWindowLongPtrW},
};

pub const PRODUCT_MARKER: usize = 0x54585056;
fn password_style(class: &str, style: u32) -> bool {
    (class.eq_ignore_ascii_case("edit") || class.to_ascii_lowercase().starts_with("richedit"))
        && style & ES_PASSWORD as u32 != 0
}
fn compatibility_interval(class: &str, mode: InsertionMode) -> Duration {
    if mode == InsertionMode::Type && class.eq_ignore_ascii_case("RichEditD2DPT") {
        Duration::from_millis(100)
    } else {
        Duration::ZERO
    }
}

// Native rich paste requires separate physical qualification. This gate cannot
// be enabled through user settings, environment variables, or debug builds.
fn rich_clipboard_qualified() -> bool {
    false
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClipboardPlan {
    None,
    Text,
    Files,
    Rich,
    RichThenFiles,
}

fn clipboard_plan(
    mode: InsertionMode,
    files: bool,
    files_first: bool,
    text_empty: bool,
    rich_available: bool,
    qualified: bool,
) -> ClipboardPlan {
    let rich = qualified && rich_available && mode == InsertionMode::Paste && !text_empty;
    let files_initially = files && (files_first || text_empty || mode == InsertionMode::Type);
    if rich && files_initially {
        ClipboardPlan::RichThenFiles
    } else if rich {
        ClipboardPlan::Rich
    } else if files_initially {
        ClipboardPlan::Files
    } else if mode == InsertionMode::Paste && !text_empty {
        ClipboardPlan::Text
    } else {
        ClipboardPlan::None
    }
}

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
    strict_clipboard: bool,
    rich_html: Option<Arc<str>>,
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
        let focused = windows::Win32::Foundation::HWND(target.focused_window as *mut _);
        let mut class = [0u16; 256];
        let length = unsafe { GetClassNameW(focused, &mut class) };
        if length <= 0 {
            return Err(ProductError::Unavailable);
        }
        let class = String::from_utf16_lossy(&class[..length as usize]);
        let style = unsafe {
            GetWindowLongPtrW(
                windows::Win32::Foundation::HWND(target.focused_window as *mut _),
                GWL_STYLE,
            )
        } as u32;
        if password_style(&class, style) {
            return Err(ProductError::Unavailable);
        }
        let minimum_interval = compatibility_interval(&class, settings.insertion);
        Ok(Self {
            target,
            generation,
            ticket,
            settings,
            armed_cancel: Cell::new(armed_cancel),
            next_send: Instant::now(),
            minimum_interval,
            board: None,
            lease: None,
            files: Default::default(),
            strict_clipboard: false,
            rich_html: None,
            submitted_events: 0,
            clipboard_incomplete: false,
        })
    }
    pub fn with_files(mut self, files: super::payload::FileDelivery) -> Self {
        self.files = files;
        self
    }
    pub fn require_complete_clipboard(mut self) -> Self {
        self.strict_clipboard = true;
        self
    }
    fn paste_shortcut(&mut self) -> Result<(), ProductError> {
        self.board
            .as_ref()
            .ok_or(ProductError::Clipboard)?
            .ensure_owned(self.lease.as_ref().ok_or(ProductError::Clipboard)?)?;
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
            if let Some(complete) = board.finish(lease)? {
                self.clipboard_incomplete |= !complete;
            }
        }
        Ok(())
    }
    pub fn deliver(&mut self, expansion: &Expansion) -> Result<(), ProductError> {
        self.rich_html = if rich_clipboard_qualified() {
            expansion.rule.rich_html.clone()
        } else {
            None
        };
        effects::wait_for_modifier_release(
            || self.check(),
            || thread::sleep(Duration::from_millis(10)),
        )?;
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
        let plan = clipboard_plan(
            mode,
            !self.files.files.is_empty(),
            self.files.first,
            text.is_empty(),
            self.rich_html.is_some(),
            self.rich_html.is_some(),
        );
        if plan != ClipboardPlan::None {
            let board = PasteBoard::new()?;
            let initial_files = if self.files.strict && !self.files.files.is_empty() {
                &self.files.files[..1]
            } else {
                &self.files.files[..]
            };
            let lease = match plan {
                ClipboardPlan::Files => board.begin_files(
                    initial_files,
                    self.settings.restore_clipboard,
                    self.strict_clipboard,
                )?,
                ClipboardPlan::Rich | ClipboardPlan::RichThenFiles => board.begin_rich_text(
                    self.rich_html.as_deref().ok_or(ProductError::Clipboard)?,
                    text,
                    self.settings.restore_clipboard,
                    self.strict_clipboard,
                )?,
                ClipboardPlan::Text => board.begin_text(
                    text,
                    self.settings.restore_clipboard,
                    self.strict_clipboard,
                )?,
                ClipboardPlan::None => unreachable!(),
            };
            self.clipboard_incomplete = lease.incomplete;
            self.board = Some(board);
            self.lease = Some(lease);
            if plan == ClipboardPlan::RichThenFiles {
                // Validate and publish both representations before erasing the
                // trigger. Store the lease first so failed preparation restores it.
                self.board
                    .as_ref()
                    .ok_or(ProductError::Clipboard)?
                    .replace_files(
                        self.lease.as_mut().ok_or(ProductError::Clipboard)?,
                        initial_files,
                    )?;
            }
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
                    let board = self.board.as_ref().ok_or(ProductError::Clipboard)?;
                    let lease = self.lease.as_mut().ok_or(ProductError::Clipboard)?;
                    if let Some(html) = &self.rich_html {
                        board.replace_rich_text(lease, html, text)?;
                    } else {
                        board.replace_text(lease, text)?;
                    }
                }
                self.settle(Duration::from_millis(25))?;
                self.paste_shortcut()?;
                self.settle(Duration::from_millis(200))?;
                self.board
                    .as_ref()
                    .ok_or(ProductError::Clipboard)?
                    .ensure_owned(self.lease.as_ref().ok_or(ProductError::Clipboard)?)
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
    let mut native = NativeEdit::new(target, generation, 1, settings)?.require_complete_clipboard();
    println!(
        "Product compatibility interval: {} ms",
        native.minimum_interval.as_millis()
    );
    // Debug fixture experiment only; never changes normal product settings.
    #[cfg(debug_assertions)]
    if let Ok(value) = std::env::var("TEXPIO_TRIAL_INTERVAL_MS") {
        let milliseconds = value
            .parse::<u64>()
            .ok()
            .filter(|value| *value <= 100)
            .ok_or(ProductError::InvalidSettings)?;
        native.minimum_interval = native
            .minimum_interval
            .max(Duration::from_millis(milliseconds));
        println!("Diagnostic minimum interval: {milliseconds} ms");
    }
    let expansion = Expansion {
        rule: ProductRule {
            id: "fixture".into(),
            trigger: "x".repeat(delete_count),
            body: output.into(),
            rich_html: None,
        },
        erase_count: delete_count,
        terminator: None,
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
    fn rich_clipboard_policy_stays_disabled_and_file_order_prepares_before_erase() {
        assert!(!rich_clipboard_qualified());
        for mode in [InsertionMode::Paste, InsertionMode::Type] {
            for (files, first, text_empty) in [
                (false, false, false),
                (true, false, false),
                (true, true, false),
                (true, true, true),
            ] {
                let plan = clipboard_plan(
                    mode,
                    files,
                    first,
                    text_empty,
                    true,
                    rich_clipboard_qualified(),
                );
                assert!(!matches!(
                    plan,
                    ClipboardPlan::Rich | ClipboardPlan::RichThenFiles
                ));
            }
        }
        assert_eq!(
            clipboard_plan(InsertionMode::Paste, false, false, false, true, true),
            ClipboardPlan::Rich
        );
        assert_eq!(
            clipboard_plan(InsertionMode::Paste, true, false, false, true, true),
            ClipboardPlan::Rich
        );
        assert_eq!(
            clipboard_plan(InsertionMode::Paste, true, true, false, true, true),
            ClipboardPlan::RichThenFiles
        );
        assert_eq!(
            clipboard_plan(InsertionMode::Type, true, true, false, true, true),
            ClipboardPlan::Files
        );
        assert_eq!(
            clipboard_plan(InsertionMode::Paste, true, true, true, true, true),
            ClipboardPlan::Files
        );
        assert_eq!(
            clipboard_plan(InsertionMode::Paste, false, false, false, false, true),
            ClipboardPlan::Text
        );
        assert_eq!(
            clipboard_plan(InsertionMode::Type, false, false, false, true, true),
            ClipboardPlan::None
        );
    }
    #[test]
    fn modern_notepad_type_is_paced_without_slowing_paste_or_classic_edit() {
        assert_eq!(
            compatibility_interval("RichEditD2DPT", InsertionMode::Type),
            Duration::from_millis(100)
        );
        assert_eq!(
            compatibility_interval("RichEditD2DPT", InsertionMode::Paste),
            Duration::ZERO
        );
        assert_eq!(
            compatibility_interval("EDIT", InsertionMode::Type),
            Duration::ZERO
        );
    }
    #[test]
    fn password_style_bit_is_only_interpreted_for_edit_controls() {
        assert!(password_style("Edit", ES_PASSWORD as u32));
        assert!(password_style("RICHEDIT50W", ES_PASSWORD as u32));
        assert!(!password_style("CustomWidget", ES_PASSWORD as u32));
        assert!(!password_style("Edit", 0));
    }
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
