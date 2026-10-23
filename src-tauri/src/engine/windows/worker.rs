use super::{
    TargetStamp,
    context::{PreparedEdit, TargetToken, WindowsContext},
    current_stamp,
    observer::{EligibleContext, ObserverHandle, ObserverShared},
    unicode::{self, InjectionPlan},
};
use crate::engine::{
    control::GateSnapshot,
    fixtures::fixtures,
    handle::SharedState,
    runtime::{Backend, DispatchCounts, ObservedCandidate, RuntimeCore},
    types::{ReferenceProfile, RefusalReason},
};
use std::{
    sync::{
        Arc,
        atomic::Ordering,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub(crate) struct WindowsJob {
    shared: Arc<ObserverShared>,
    ready: Receiver<Result<(), RefusalReason>>,
    worker: Option<JoinHandle<()>>,
    observer: Option<ObserverHandle>,
}

struct CloseOnExit(Arc<ObserverShared>);
impl Drop for CloseOnExit {
    fn drop(&mut self) {
        self.0.gate.close();
        self.0.halt.store(true, Ordering::SeqCst);
    }
}

impl WindowsJob {
    pub fn spawn(
        state: Arc<SharedState>,
        profile: ReferenceProfile,
    ) -> Result<Self, RefusalReason> {
        let (sender, receiver) = mpsc::sync_channel(128);
        let shared = Arc::new(ObserverShared::new(state.gate.clone(), sender));
        let worker_shared = shared.clone();
        let (ready_sender, ready) = mpsc::sync_channel(1);
        let worker = thread::Builder::new()
            .name("texpio-context".into())
            .spawn(move || {
                let context = match WindowsContext::new() {
                    Ok(context) => context,
                    Err(reason) => {
                        state.fail(reason, false);
                        let _ = ready_sender.send(Err(reason));
                        return;
                    }
                };
                let backend = WindowsBackend {
                    context,
                    target: None,
                    profile,
                    shared: worker_shared.clone(),
                };
                let mut core = RuntimeCore::new(state.gate.clone(), backend);
                let _close_on_exit = CloseOnExit(worker_shared.clone());
                if ready_sender.send(Ok(())).is_err() {
                    state.gate.close();
                    return;
                }
                let mut refreshed = Instant::now() - Duration::from_secs(1);
                while state.gate.is_active() && !worker_shared.halt.load(Ordering::SeqCst) {
                    if refreshed.elapsed() >= Duration::from_millis(100) {
                        let result = core.backend_mut().refresh();
                        state.reason(result.err());
                        refreshed = Instant::now();
                    }
                    if let Some(reason) = worker_shared.take_reason() {
                        state.reason(Some(reason));
                    }
                    match receiver.recv_timeout(Duration::from_millis(20)) {
                        Ok(request) => state.record(core.process(&request)),
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                worker_shared.halt.store(true, Ordering::SeqCst);
                state.gate.close();
                if let Ok(mut eligible) = worker_shared.eligible.lock() {
                    *eligible = None;
                }
            })
            .map_err(|_| RefusalReason::ContextUnavailable)?;
        Ok(Self {
            shared,
            ready,
            worker: Some(worker),
            observer: None,
        })
    }

    pub fn initialize(&mut self) -> Result<(), RefusalReason> {
        self.ready
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| RefusalReason::ProviderTimeout)??;
        if !self.shared.gate.is_active() {
            return Err(RefusalReason::Stopped);
        }
        self.observer = Some(ObserverHandle::start(self.shared.clone())?);
        if !self.shared.gate.is_active() {
            return Err(RefusalReason::Stopped);
        }
        Ok(())
    }
    pub fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
    pub fn shutdown(&mut self) -> Result<(), RefusalReason> {
        self.shared.gate.close();
        self.shared.halt.store(true, Ordering::SeqCst);
        let clean = self.observer.as_mut().is_none_or(ObserverHandle::stop);
        let deadline = Instant::now() + Duration::from_millis(500);
        while !self.finished() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(5));
        }
        if !self.finished() {
            return Err(RefusalReason::ProviderTimeout);
        }
        if let Some(worker) = self.worker.take()
            && worker.join().is_err()
        {
            return Err(RefusalReason::ContextUnavailable);
        }
        if clean {
            Ok(())
        } else {
            Err(RefusalReason::ContextUnavailable)
        }
    }
}

impl Drop for WindowsJob {
    fn drop(&mut self) {
        self.shared.gate.close();
        self.shared.halt.store(true, Ordering::SeqCst);
        if let Some(observer) = self.observer.as_mut() {
            let _ = observer.stop();
        }
    }
}

struct WindowsBackend {
    context: WindowsContext,
    target: Option<(u64, TargetStamp, TargetToken)>,
    profile: ReferenceProfile,
    shared: Arc<ObserverShared>,
}
struct NativeEdit {
    edit: PreparedEdit,
    plan: InjectionPlan,
}

impl WindowsBackend {
    fn refresh(&mut self) -> Result<(), RefusalReason> {
        let origin = current_stamp()?;
        let generation = self.shared.gate.snapshot().context;
        if let Some((current, stamp, token)) = &self.target
            && *current == generation
            && *stamp == origin
            && self.context.recheck_target(token).is_ok()
        {
            return Ok(());
        }
        self.target = None;
        if let Ok(mut eligible) = self.shared.eligible.lock() {
            *eligible = None;
        }
        self.shared.gate.invalidate_context();
        let generation = self.shared.gate.snapshot().context;
        let target = self.context.capture(self.profile, &origin)?;
        if !self.shared.gate.is_active()
            || generation != self.shared.gate.snapshot().context
            || current_stamp()? != origin
        {
            return Err(RefusalReason::StaleContext);
        }
        self.target = Some((generation, origin, target));
        *self
            .shared
            .eligible
            .lock()
            .map_err(|_| RefusalReason::ContextUnavailable)? = Some(EligibleContext {
            stamp: origin,
            generation,
        });
        Ok(())
    }
}

impl Backend for WindowsBackend {
    type Origin = TargetStamp;
    type Edit = NativeEdit;
    fn prepare(
        &mut self,
        request: &ObservedCandidate<TargetStamp>,
    ) -> Result<NativeEdit, RefusalReason> {
        let (generation, origin, target) =
            self.target.as_ref().ok_or(RefusalReason::StaleContext)?;
        if *generation != request.ticket.context || *origin != request.origin {
            return Err(RefusalReason::StaleContext);
        }
        let fixture = fixtures()
            .into_iter()
            .find(|fixture| {
                fixture.id == request.candidate.fixture_id
                    && fixture.trigger == request.candidate.trigger
            })
            .ok_or(RefusalReason::ContextUnavailable)?;
        let edit = self
            .context
            .prepare_automatic(target, &request.candidate, &fixture.body)?;
        let plan = unicode::unicode_plan(&edit)?;
        Ok(NativeEdit { edit, plan })
    }
    fn revalidate(&mut self, edit: &NativeEdit) -> Result<(), RefusalReason> {
        self.context.revalidate(&edit.edit)
    }
    fn dispatch(&mut self, edit: &NativeEdit) -> Result<DispatchCounts, RefusalReason> {
        unicode::dispatch(&edit.edit, &edit.plan)
    }
    fn verify(&mut self, edit: &NativeEdit, ticket: GateSnapshot) -> Result<bool, RefusalReason> {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            self.shared.gate.validate(ticket)?;
            if self.context.verify(&edit.edit)? {
                self.shared.gate.validate(ticket)?;
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(5));
        }
        Ok(false)
    }
}
