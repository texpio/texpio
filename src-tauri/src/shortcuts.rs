//! Shortcut ownership is serialized on a worker. No application mutex is held while
//! the plugin dispatches registration/unregistration to the native main thread.
use crate::types::ServiceError;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc,
};
pub const DEFAULT_SHORTCUT: &str = "CommandOrControl+Shift+.";
pub trait ShortcutBackend: Send + 'static {
    fn canonicalize(&self, chord: &str) -> Result<String, ServiceError>;
    fn register(&mut self, chord: &str, generation: u64) -> Result<(), ServiceError>;
    fn unregister(&mut self, chord: &str) -> Result<(), ServiceError>;
    fn load(&self) -> Result<Option<String>, ServiceError>;
    fn save(&mut self, chord: Option<String>) -> Result<(), ServiceError>;
}
pub struct ShortcutController<B: ShortcutBackend> {
    backend: B,
    current: Option<String>,
    saved: Option<String>,
    owned: Vec<String>,
    generation: u64,
    active: Arc<AtomicU64>,
    dirty: bool,
}
impl<B: ShortcutBackend> ShortcutController<B> {
    pub fn new(backend: B, active: Arc<AtomicU64>) -> Self {
        Self {
            backend,
            current: None,
            saved: None,
            owned: vec![],
            generation: 0,
            active,
            dirty: false,
        }
    }
    fn next(&mut self) -> Result<u64, ServiceError> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(ServiceError::Unavailable)?;
        Ok(self.generation)
    }
    pub fn initialize(&mut self) -> Result<(), ServiceError> {
        self.saved = self.backend.load()?;
        let chord = self
            .backend
            .canonicalize(self.saved.as_deref().unwrap_or(DEFAULT_SHORTCUT))?;
        let generation = self.next()?;
        self.backend.register(&chord, generation)?;
        self.owned.push(chord.clone());
        self.current = Some(chord);
        self.active.store(generation, Ordering::Release);
        Ok(())
    }
    fn release(&mut self, chord: &str) -> Result<(), ServiceError> {
        self.backend.unregister(chord)?;
        self.owned.retain(|owned| owned != chord);
        Ok(())
    }
    pub fn replace(&mut self, input: &str) -> Result<(), ServiceError> {
        let chord = self.backend.canonicalize(input)?;
        // Failed rollback resources are kept inactive and retried before any new binding.
        for orphan in self.owned.clone() {
            if self.current.as_ref() != Some(&orphan) {
                self.release(&orphan)?;
            }
        }
        if self.dirty {
            self.backend.save(self.saved.clone())?;
            self.dirty = false;
        }
        if self.current.as_ref() == Some(&chord) {
            return Ok(());
        }
        let previous_saved = self.backend.load()?;
        let generation = self.next()?;
        self.backend.register(&chord, generation)?;
        self.owned.push(chord.clone());
        if let Err(error) = self.backend.save(Some(chord.clone())) {
            let _ = self.release(&chord);
            return Err(error);
        }
        if let Some(old) = self.current.clone()
            && self.release(&old).is_err()
        {
            self.saved = previous_saved.clone();
            self.dirty = self.backend.save(previous_saved).is_err();
            let _ = self.release(&chord);
            return Err(ServiceError::Unavailable);
        }
        self.saved = Some(chord.clone());
        self.current = Some(chord);
        self.active.store(generation, Ordering::Release);
        Ok(())
    }
}
type Request = (String, mpsc::SyncSender<Result<(), ServiceError>>);
#[derive(Clone)]
pub struct ShortcutActor {
    sender: mpsc::SyncSender<Request>,
}
impl ShortcutActor {
    pub fn spawn<B: ShortcutBackend>(backend: B, active: Arc<AtomicU64>) -> Self {
        let (sender, receiver) = mpsc::sync_channel::<Request>(16);
        std::thread::spawn(move || {
            let mut controller = ShortcutController::new(backend, active);
            // A conflict or corrupt preference disables only this feature, never app/tray startup.
            let _ = controller.initialize();
            while let Ok((chord, reply)) = receiver.recv() {
                let _ = reply.send(controller.replace(&chord));
            }
        });
        Self { sender }
    }
    pub fn replace(&self, chord: String) -> Result<(), ServiceError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        self.sender
            .try_send((chord, sender))
            .map_err(|_| ServiceError::Unavailable)?;
        receiver.recv().map_err(|_| ServiceError::Unavailable)?
    }
}
