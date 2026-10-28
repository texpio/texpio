//! Native registration runs on a serialized worker; callbacks never take its locks.
use super::{
    desktop_settings::{DesktopAction, DesktopBackend, DesktopController},
    effects::ProductError,
    service::{ProductEngine, ProductPhase},
    settings::ProductSettings,
    store::SettingsStore,
};
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc,
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
struct Published {
    closed: AtomicBool,
    epoch: AtomicU64,
    keys: RwLock<Vec<(DesktopAction, String)>>,
    toggle: AtomicBool,
    clipboard: AtomicBool,
}
impl Published {
    fn pending(&self, action: DesktopAction) -> &AtomicBool {
        match action {
            DesktopAction::Toggle => &self.toggle,
            DesktopAction::Clipboard => &self.clipboard,
        }
    }
}
struct Pending {
    published: Arc<Published>,
    action: DesktopAction,
}
impl Drop for Pending {
    fn drop(&mut self) {
        self.published
            .pending(self.action)
            .store(false, Ordering::Release);
    }
}
struct Native {
    app: AppHandle,
    settings: Arc<SettingsStore>,
    published: Arc<Published>,
}
impl DesktopBackend for Native {
    type Startup = super::startup::StartupValue;
    fn canonicalize(&self, chord: &str) -> Result<String, ProductError> {
        if chord.len() > 128
            || !chord.is_ascii()
            || !chord.split('+').any(|part| {
                matches!(
                    part.to_ascii_lowercase().as_str(),
                    "commandorcontrol"
                        | "cmdorctrl"
                        | "ctrl"
                        | "control"
                        | "cmd"
                        | "command"
                        | "alt"
                        | "super"
                        | "meta"
                )
            })
        {
            return Err(ProductError::InvalidSettings);
        }
        let shortcut = chord
            .parse::<Shortcut>()
            .map_err(|_| ProductError::InvalidSettings)?
            .to_string();
        let quick = self
            .app
            .state::<Arc<crate::commands::preferences::PreferencesStore>>()
            .load()
            .map_err(|_| ProductError::Unavailable)?
            .quick_pane_shortcut
            .unwrap_or_else(|| crate::shortcuts::DEFAULT_SHORTCUT.into())
            .parse::<Shortcut>()
            .map_err(|_| ProductError::InvalidSettings)?
            .to_string();
        if shortcut == quick {
            return Err(ProductError::InvalidSettings);
        }
        Ok(shortcut)
    }
    fn register(&mut self, chord: &str) -> Result<(), ProductError> {
        let key = chord.to_owned();
        let published = self.published.clone();
        self.app
            .global_shortcut()
            .on_shortcut(chord, move |app, _, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                let selected = {
                    let keys = published.keys.read().unwrap_or_else(|p| p.into_inner());
                    keys.iter()
                        .find(|(_, chord)| chord == &key)
                        .map(|(action, _)| (*action, published.epoch.load(Ordering::Acquire)))
                };
                if let Some((action, epoch)) = selected {
                    dispatch(app.clone(), published.clone(), action, epoch);
                }
            })
            .map_err(|_| ProductError::ShortcutUnavailable)
    }
    fn unregister(&mut self, chord: &str) -> Result<(), ProductError> {
        self.app
            .global_shortcut()
            .unregister(chord)
            .map_err(|_| ProductError::Unavailable)
    }
    fn startup_snapshot(&self) -> Result<Self::Startup, ProductError> {
        super::startup::snapshot(&self.app.config().identifier)
    }
    fn startup(&mut self, enabled: bool) -> Result<(), ProductError> {
        super::startup::set(&self.app.config().identifier, enabled)
    }
    fn restore_startup(&mut self, value: Self::Startup) -> Result<(), ProductError> {
        super::startup::restore(&self.app.config().identifier, value)
    }
    fn save(&mut self, settings: &ProductSettings) -> Result<(), ProductError> {
        self.settings.save(settings)
    }
    fn publish(&mut self, keys: &[(DesktopAction, String)]) {
        let mut current = self
            .published
            .keys
            .write()
            .unwrap_or_else(|p| p.into_inner());
        self.published.epoch.fetch_add(1, Ordering::AcqRel);
        *current = if self.published.closed.load(Ordering::Acquire) {
            vec![]
        } else {
            keys.to_vec()
        };
    }
}
enum Request {
    Apply(
        ProductSettings,
        mpsc::SyncSender<Result<ProductSettings, ProductError>>,
    ),
    Stop,
}
#[derive(Clone)]
pub struct DesktopSettingsActor {
    sender: mpsc::SyncSender<Request>,
    published: Arc<Published>,
}
impl DesktopSettingsActor {
    pub fn start(app: &AppHandle) -> Self {
        let published = Arc::new(Published {
            closed: AtomicBool::new(false),
            epoch: AtomicU64::new(0),
            keys: RwLock::new(vec![]),
            toggle: AtomicBool::new(false),
            clipboard: AtomicBool::new(false),
        });
        let (sender, receiver) = mpsc::sync_channel(1);
        let backend = Native {
            app: app.clone(),
            settings: app.state::<Arc<SettingsStore>>().inner().clone(),
            published: published.clone(),
        };
        let _ = std::thread::Builder::new()
            .name("texpio-desktop-settings".into())
            .spawn(move || {
                let initial = backend.settings.load();
                let app = backend.app.clone();
                let mut controller = DesktopController::new(backend);
                match initial {
                    Ok(settings) => {
                        let result = controller.initialize_session(settings, || {
                            let engine = app.state::<Arc<ProductEngine>>();
                            let service =
                                app.state::<Arc<crate::library_commands::LibraryService>>();
                            let probe = app.state::<Arc<crate::engine::runtime::ProbeHandle>>();
                            let store = app.state::<Arc<SettingsStore>>();
                            service
                                .start_product_on_launch(&engine, &probe, &store)
                                .map(|_| ())
                        });
                        if result != Err(ProductError::Cancelled) {
                            report(&app, result, None);
                        }
                    }
                    Err(error) => report(&app, Err(error), None),
                }
                while let Ok(request) = receiver.recv() {
                    match request {
                        Request::Apply(settings, reply) => {
                            let _ = reply.send(controller.apply(settings));
                        }
                        Request::Stop => {
                            controller.shutdown();
                            return;
                        }
                    }
                }
                controller.shutdown();
            });
        Self { sender, published }
    }
    pub fn apply(&self, settings: ProductSettings) -> Result<ProductSettings, ProductError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        self.sender
            .try_send(Request::Apply(settings, sender))
            .map_err(|_| ProductError::Busy)?;
        receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| ProductError::Unavailable)?
    }
    pub fn shutdown(&self) {
        self.published.closed.store(true, Ordering::Release);
        self.published.epoch.fetch_add(1, Ordering::AcqRel);
        self.published
            .keys
            .write()
            .unwrap_or_else(|p| p.into_inner())
            .clear();
        let _ = self.sender.try_send(Request::Stop);
    }
}
fn report(
    app: &AppHandle,
    result: Result<(), ProductError>,
    resume_failure: Option<(u64, ProductError)>,
) {
    if let Some(engine) = app.try_state::<Arc<ProductEngine>>() {
        engine.record_desktop_result(result, resume_failure);
    }
    crate::desktop::refresh_probe_tray(app);
    let _ = app.emit("product-status-changed", ());
}
fn dispatch(app: AppHandle, published: Arc<Published>, action: DesktopAction, epoch: u64) {
    if published.pending(action).swap(true, Ordering::AcqRel) {
        return;
    }
    let Some(engine) = app
        .try_state::<Arc<ProductEngine>>()
        .map(|engine| engine.inner().clone())
    else {
        published.pending(action).store(false, Ordering::Release);
        return;
    };
    let admission = engine.admission();
    let target = if action == DesktopAction::Clipboard {
        crate::engine::windows::current_stamp().ok()
    } else {
        None
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _pending = Pending {
            published: published.clone(),
            action,
        };
        let mut resume_failure = None;
        let result = (|| {
            if published.epoch.load(Ordering::Acquire) != epoch || engine.admission() != admission {
                return Err(ProductError::Cancelled);
            }
            let settings = app.state::<Arc<SettingsStore>>().load()?;
            let service = app.state::<Arc<crate::library_commands::LibraryService>>();
            match action {
                DesktopAction::Toggle => {
                    if engine.status().phase == ProductPhase::Running
                        || engine.shared.busy.load(Ordering::Acquire)
                    {
                        engine.stop();
                        Ok(())
                    } else {
                        let probe = app.state::<Arc<crate::engine::runtime::ProbeHandle>>();
                        let store = app.state::<Arc<SettingsStore>>();
                        service
                            .start_product_admitted(&engine, &probe, &store, admission)
                            .map(|_| ())
                    }
                }
                DesktopAction::Clipboard => {
                    let target = target.ok_or(ProductError::Unavailable)?;
                    if target.process_id == std::process::id() {
                        return Err(ProductError::Unavailable);
                    }
                    clipboard(
                        &app,
                        &engine,
                        &service,
                        settings,
                        target,
                        admission,
                        (&published, epoch, &mut resume_failure),
                    )
                }
            }
        })();
        report(&app, result, resume_failure);
    });
}
fn clipboard(
    app: &AppHandle,
    engine: &ProductEngine,
    service: &crate::library_commands::LibraryService,
    settings: ProductSettings,
    target: crate::engine::windows::TargetStamp,
    admission: u64,
    current: (&Published, u64, &mut Option<(u64, ProductError)>),
) -> Result<(), ProductError> {
    let (published, epoch, resume_failure) = current;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
    loop {
        if unsafe {
            GetAsyncKeyState(i32::from(
                settings.cancel_vk().ok_or(ProductError::InvalidSettings)?,
            )) < 0
        } {
            return Err(ProductError::Cancelled);
        }
        if published.epoch.load(Ordering::Acquire) != epoch || engine.admission() != admission {
            return Err(ProductError::Cancelled);
        }
        if crate::engine::windows::current_stamp().ok() != Some(target) {
            return Err(ProductError::TargetChanged);
        }
        if ![VK_SHIFT, VK_CONTROL, VK_MENU, VK_LWIN, VK_RWIN]
            .iter()
            .any(|key| unsafe { GetAsyncKeyState(i32::from(key.0)) < 0 })
        {
            break;
        }
        if std::time::Instant::now() >= deadline {
            return Err(ProductError::ModifiersHeld);
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    service.coordinated(|| {
        if published.epoch.load(Ordering::Acquire) != epoch || engine.admission() != admission {
            return Err(ProductError::Cancelled);
        }
        let store = app.state::<Arc<SettingsStore>>();
        let settings = store.load()?;
        let probe = app.state::<Arc<crate::engine::runtime::ProbeHandle>>();
        if probe.stop().phase != crate::engine::types::ProbePhase::Stopped {
            return Err(ProductError::Busy);
        }
        let (resume, input) = engine.suspend_admitted(admission)?;
        let result = (|| {
            engine
                .shared
                .busy
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .map_err(|_| ProductError::Busy)?;
            struct Busy<'a>(&'a AtomicBool);
            impl Drop for Busy<'_> {
                fn drop(&mut self) {
                    self.0.store(false, Ordering::Release);
                }
            }
            let _busy = Busy(&engine.shared.busy);
            let _effects = engine
                .shared
                .effects
                .try_lock()
                .map_err(|_| ProductError::Busy)?;
            let board = super::clipboard::PasteBoard::new()?;
            let text = board.text()?.ok_or(ProductError::Clipboard)?;
            if text.chars().take(1_000_001).count() > 1_000_000
                || text
                    .chars()
                    .any(|ch| ch.is_control() && !matches!(ch, '\r' | '\n' | '\t'))
            {
                return Err(ProductError::Clipboard);
            }
            let mut clipboard_settings = settings.clone();
            clipboard_settings.insertion = settings.clipboard_mode;
            clipboard_settings.preserve_terminator = false;
            let mut edit = super::windows::NativeEdit::new(
                target,
                engine.shared.generation.clone(),
                input,
                clipboard_settings,
            )?;
            let expansion = super::matcher::Expansion {
                rule: super::matcher::ProductRule {
                    id: "clipboard".into(),
                    trigger: String::new(),
                    body: text.into(),
                    rich_html: None,
                },
                erase_count: 0,
                terminator: None,
            };
            let result = edit.deliver(&expansion);
            let mut status = engine
                .shared
                .status
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            status.submitted = status
                .submitted
                .saturating_add(u32::from(edit.submitted_events > 0));
            status.clipboard_incomplete = edit.clipboard_incomplete;
            result
        })();
        if let Some(ticket) = resume
            && let Err(error) = service.resume_product_locked(engine, &store, ticket)
        {
            *resume_failure = Some((ticket, error));
        }
        result
    })?
}
