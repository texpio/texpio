//! AT-SPI owns field identity; X focus identifies only a client window. Queries run
//! on one dedicated thread, so capture/effects share a strong focused-object epoch.
use crate::product::effects::ProductError;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::{thread, time::Duration};

#[derive(Clone, Debug)]
pub(super) struct Evidence {
    pub token: u64,
    pub caret: i32,
    pub suffix: String,
}
struct Request {
    pid: u32,
    generation: u64,
    reply: mpsc::SyncSender<Result<Evidence, ProductError>>,
}
struct Inner {
    requests: mpsc::SyncSender<Request>,
    stopped: Arc<AtomicBool>,
    done: Mutex<mpsc::Receiver<()>>,
    thread: Mutex<Option<thread::JoinHandle<()>>>,
}
impl Inner {
    fn shutdown(&self) -> bool {
        self.stopped.store(true, Ordering::SeqCst);
        let Ok(mut thread) = self.thread.lock() else {
            return false;
        };
        if thread.is_none() {
            return true;
        }
        let acknowledged = self
            .done
            .lock()
            .is_ok_and(|done| done.recv_timeout(Duration::from_millis(800)).is_ok());
        if acknowledged && let Some(thread) = thread.take() {
            let _ = thread.join();
        }
        thread.is_none()
    }
}
impl Drop for Inner {
    fn drop(&mut self) {
        self.shutdown();
    }
}
#[derive(Clone)]
pub(super) struct Guard(Arc<Inner>);
impl Guard {
    pub fn start() -> Result<Self, ProductError> {
        let (requests, incoming) = mpsc::sync_channel::<Request>(1);
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = stopped.clone();
        let (done_tx, done) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("texpio-x11-context".into())
            .spawn(move || {
                #[cfg(target_os = "linux")]
                let mut tracker = texpio_wayland_helper::linux::FieldTracker::new();
                while !stop.load(Ordering::SeqCst) {
                    let request = match incoming.recv_timeout(Duration::from_millis(25)) {
                        Ok(request) => request,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(_) => break,
                    };
                    #[cfg(target_os = "linux")]
                    let result = (|| {
                        if !texpio_wayland_helper::linux::session_unlocked() {
                            return Err(ProductError::InputBlocked);
                        }
                        let native = tracker
                            .as_mut()
                            .map_err(|_| ProductError::Unavailable)?
                            .snapshot(request.pid, request.generation)
                            .map_err(|_| ProductError::InputBlocked)?;
                        // AT-SPI has no universal preedit contract. Unknown stays Unknown;
                        // an environment variable or XKB key symbol is not Idle evidence.
                        if !native.safe() {
                            return Err(ProductError::InputBlocked);
                        }
                        Ok(Evidence {
                            token: native.token,
                            caret: native.caret,
                            suffix: native.suffix,
                        })
                    })();
                    #[cfg(not(target_os = "linux"))]
                    let result = {
                        let _ = (request.pid, request.generation);
                        Err(ProductError::Unavailable)
                    };
                    let _ = request.reply.send(result);
                }
                let _ = done_tx.send(());
            })
            .map_err(|_| ProductError::Unavailable)?;
        Ok(Self(Arc::new(Inner {
            requests,
            stopped,
            done: Mutex::new(done),
            thread: Mutex::new(Some(worker)),
        })))
    }
    pub fn snapshot(&self, pid: u32, generation: u64) -> Result<Evidence, ProductError> {
        let (reply, result) = mpsc::sync_channel(1);
        self.0
            .requests
            .try_send(Request {
                pid,
                generation,
                reply,
            })
            .map_err(|_| ProductError::Busy)?;
        result
            .recv_timeout(Duration::from_millis(350))
            .map_err(|_| ProductError::Unavailable)?
    }
    pub fn shutdown(&self) -> bool {
        self.0.shutdown()
    }
}

pub(super) fn provider_ready() -> Result<(), ProductError> {
    // Deliberate functional gate. FieldTracker currently reports Composition::Unknown;
    // no direct-input toolkit provider has supplied positive composition evidence yet.
    // Neither startup nor a status read may arm an unqualified observer as Running.
    Err(ProductError::InputBlocked)
}
