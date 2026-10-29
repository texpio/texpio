#[cfg(any(target_os = "windows", test))]
use super::runtime::OperationResult;
#[cfg(target_os = "windows")]
use super::windows::worker::WindowsJob;
use super::{
    control::{CloseOutcome, DispatchGate},
    rules::{RuleSet, TargetPolicy},
    runtime::StartOptions,
    types::{ProbePhase, ProbeStatus, ReferenceProfile, RefusalReason},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

pub(crate) struct SharedState {
    pub gate: Arc<DispatchGate>,
    status: Mutex<ProbeStatus>,
    activity: Mutex<Option<Instant>>,
    uncertain: AtomicBool,
}

impl SharedState {
    pub fn snapshot(&self) -> ProbeStatus {
        let mut status = self
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        if status.phase == ProbePhase::Running && !self.gate.is_active() {
            status.phase = ProbePhase::Fault;
            status.refusal_reason = Some(RefusalReason::StaleContext);
        }
        status.last_activity_age_ms = self
            .activity
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .map(|time| time.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
        status
    }
    pub fn fail(&self, reason: RefusalReason, uncertain: bool) {
        self.gate.close();
        let mut status = self.status.lock().unwrap_or_else(|p| p.into_inner());
        status.phase = ProbePhase::Fault;
        status.refusal_reason = Some(reason);
        if uncertain && !self.uncertain.swap(true, Ordering::SeqCst) {
            status.counters.uncertain += 1;
        }
    }
    #[cfg(target_os = "windows")]
    pub fn reason(&self, reason: Option<RefusalReason>) {
        let mut status = self.status.lock().unwrap_or_else(|p| p.into_inner());
        if status.phase == ProbePhase::Running {
            status.refusal_reason = reason;
        }
    }
    #[cfg(any(target_os = "windows", test))]
    pub fn record(&self, result: OperationResult) {
        *self.activity.lock().unwrap_or_else(|p| p.into_inner()) = Some(Instant::now());
        if matches!(result, OperationResult::Uncertain { .. }) {
            self.fail(RefusalReason::OutcomeUncertain, true);
        }
        let mut status = self.status.lock().unwrap_or_else(|p| p.into_inner());
        match result {
            OperationResult::Observed => {
                status.counters.submitted += 1;
                status.counters.observed += 1;
            }
            OperationResult::Uncertain { submitted } => {
                status.counters.submitted += u64::from(submitted)
            }
            OperationResult::Refused(reason) => {
                status.counters.refused += 1;
                if status.phase == ProbePhase::Running {
                    status.refusal_reason = Some(reason);
                }
            }
        }
    }
}

pub struct ProbeHandle {
    pub(crate) shared: Arc<SharedState>,
    excluded: Mutex<Vec<ReferenceProfile>>,
    session: Mutex<Option<Session>>,
    library_epoch: AtomicU64,
    publication: Mutex<u64>,
    #[cfg(target_os = "windows")]
    job: Mutex<Option<WindowsJob>>,
}

#[derive(Clone)]
struct Session {
    policy: TargetPolicy,
    #[cfg(target_os = "windows")]
    rules: Arc<RuleSet>,
    profile: Option<ReferenceProfile>,
    library_epoch: u64,
}

impl Default for ProbeHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl ProbeHandle {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(SharedState {
                gate: Arc::new(DispatchGate::new()),
                status: Mutex::new(ProbeStatus::default()),
                activity: Mutex::new(None),
                uncertain: AtomicBool::new(false),
            }),
            excluded: Mutex::new(Vec::new()),
            session: Mutex::new(None),
            library_epoch: AtomicU64::new(0),
            publication: Mutex::new(0),
            #[cfg(target_os = "windows")]
            job: Mutex::new(None),
        }
    }
    pub fn status(&self) -> ProbeStatus {
        self.shared.snapshot()
    }

    pub fn start(&self, options: StartOptions) -> Result<ProbeStatus, RefusalReason> {
        if options.strategy != super::types::InsertionStrategy::Unicode {
            return Err(RefusalReason::ClipboardState);
        }
        self.start_session(Session {
            policy: TargetPolicy::Probe(options.profile),
            #[cfg(target_os = "windows")]
            rules: Arc::new(RuleSet::fixtures()),
            profile: Some(options.profile),
            library_epoch: 0,
        })
    }

    /// Only Rust-validated, immutable library snapshots may start a product session.
    pub fn start_library(&self, rules: Arc<RuleSet>) -> Result<ProbeStatus, RefusalReason> {
        if rules.entries().is_empty() {
            return Err(RefusalReason::ContextUnavailable);
        }
        self.start_session(Session {
            policy: TargetPolicy::NotepadEditor,
            #[cfg(target_os = "windows")]
            rules,
            profile: None,
            library_epoch: self.library_epoch.load(Ordering::SeqCst),
        })
    }

    /// Invalidates even an in-flight pause's saved snapshot before content changes.
    pub fn invalidate_library(&self) -> ProbeStatus {
        self.library_epoch.fetch_add(1, Ordering::SeqCst);
        self.stop()
    }

    fn start_session(&self, session: Session) -> Result<ProbeStatus, RefusalReason> {
        #[cfg(target_os = "windows")]
        let admission = *self
            .publication
            .lock()
            .map_err(|_| RefusalReason::OutcomeUncertain)?;
        if session.policy == TargetPolicy::NotepadEditor
            && session.library_epoch != self.library_epoch.load(Ordering::SeqCst)
        {
            return Err(RefusalReason::StaleContext);
        }
        let exclusion = session.profile.unwrap_or(ReferenceProfile::Notepad);
        if self
            .excluded
            .lock()
            .map_err(|_| RefusalReason::OutcomeUncertain)?
            .contains(&exclusion)
        {
            return Err(RefusalReason::Excluded);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = session;
            Err(RefusalReason::UnsupportedPlatform)
        }
        #[cfg(target_os = "windows")]
        {
            let mut slot = self
                .job
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)?;
            if let Some(job) = slot.as_mut() {
                if !job.finished() {
                    return Err(RefusalReason::Busy);
                }
                job.shutdown()?;
            }
            *slot = None;
            self.arm_session(admission)?;
            self.shared.uncertain.store(false, Ordering::SeqCst);
            *self
                .shared
                .status
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)? = ProbeStatus {
                phase: ProbePhase::Running,
                profile: session.profile,
                strategy: super::types::InsertionStrategy::Unicode,
                ..ProbeStatus::default()
            };
            let mut job =
                match WindowsJob::spawn(self.shared.clone(), session.policy, session.rules.clone())
                {
                    Ok(job) => job,
                    Err(reason) => {
                        self.shared.fail(reason, false);
                        return Err(reason);
                    }
                };
            let ready = job.initialize();
            if let Err(reason) = ready {
                self.shared.fail(reason, false);
                let _ = job.shutdown();
                *slot = Some(job);
                return Err(reason);
            }
            *slot = Some(job);
            let publication = self
                .publication
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)?;
            if !self.shared.gate.is_active()
                || *publication != admission
                || (session.policy == TargetPolicy::NotepadEditor
                    && session.library_epoch != self.library_epoch.load(Ordering::SeqCst))
            {
                self.shared.gate.close();
                return Err(RefusalReason::Stopped);
            }
            *self
                .session
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)? = Some(session);
            Ok(self.status())
        }
    }

    #[cfg(any(target_os = "windows", test))]
    fn arm_session(&self, admission: u64) -> Result<(), RefusalReason> {
        // Only bounded in-memory activation lives under this lock. Provider calls,
        // native dispatch and worker initialization must remain outside it.
        let publication = self
            .publication
            .lock()
            .map_err(|_| RefusalReason::OutcomeUncertain)?;
        if *publication != admission {
            return Err(RefusalReason::StaleContext);
        }
        self.shared.gate.arm().map(|_| ())
    }

    pub fn stop(&self) -> ProbeStatus {
        let outcome = {
            let mut publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
            *publication = publication.wrapping_add(1);
            let outcome = self.shared.gate.close();
            *self.session.lock().unwrap_or_else(|p| p.into_inner()) = None;
            outcome
        };
        #[cfg(target_os = "windows")]
        let cleanup = match self.job.try_lock() {
            Ok(mut slot) => {
                if let Some(job) = slot.as_mut() {
                    job.shutdown()
                } else {
                    Ok(())
                }
            }
            Err(_) => Err(RefusalReason::Busy),
        };
        #[cfg(not(target_os = "windows"))]
        let cleanup: Result<(), RefusalReason> = Ok(());
        if outcome == CloseOutcome::InFlightUncertain {
            self.shared.fail(RefusalReason::OutcomeUncertain, true);
        } else if let Err(reason) = cleanup {
            self.shared.fail(reason, false);
        } else {
            let mut status = self.shared.status.lock().unwrap_or_else(|p| p.into_inner());
            status.phase = ProbePhase::Stopped;
            status.refusal_reason = None;
        }
        self.status()
    }

    pub fn set_paused(&self, paused: bool) -> Result<ProbeStatus, RefusalReason> {
        if paused {
            let admission = *self
                .publication
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)?;
            let session = self
                .session
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)?
                .clone();
            let stopped = self.stop();
            if stopped.phase == ProbePhase::Stopped {
                let publication = self
                    .publication
                    .lock()
                    .map_err(|_| RefusalReason::OutcomeUncertain)?;
                if *publication != admission.wrapping_add(1) {
                    return Err(RefusalReason::StaleContext);
                }
                self.shared
                    .status
                    .lock()
                    .map_err(|_| RefusalReason::OutcomeUncertain)?
                    .phase = ProbePhase::Paused;
                *self
                    .session
                    .lock()
                    .map_err(|_| RefusalReason::OutcomeUncertain)? = session;
            }
            Ok(self.status())
        } else {
            let session = self
                .session
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)?
                .clone()
                .ok_or(RefusalReason::Stopped)?;
            self.start_session(session)
        }
    }

    pub fn exclude(&self, profile: ReferenceProfile) -> ProbeStatus {
        let mut excluded = self.excluded.lock().unwrap_or_else(|p| p.into_inner());
        if !excluded.contains(&profile) {
            excluded.push(profile);
        }
        drop(excluded);
        // Also cancels a concurrently starting session before it publishes a profile.
        // P0 exclusions stop the current session and require explicit rearming.
        self.stop();
        if self.status().profile == Some(profile) {
            self.shared
                .status
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .refusal_reason = Some(RefusalReason::Excluded);
        }
        self.status()
    }
}

impl Drop for ProbeHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{ProbePhase, ReferenceProfile};
    use super::*;

    #[test]
    fn stop_cancels_a_start_admitted_before_native_initialization() {
        assert_cancelled_start(ProbeHandle::stop);
    }

    #[test]
    fn content_update_cancels_an_admitted_resume_before_observation_can_start() {
        assert_cancelled_start(ProbeHandle::invalidate_library);
    }

    fn assert_cancelled_start(cancel: fn(&ProbeHandle) -> ProbeStatus) {
        use std::sync::Barrier;
        let handle = Arc::new(ProbeHandle::new());
        let admission = *handle.publication.lock().unwrap();
        let ready = Arc::new(Barrier::new(2));
        let resume = Arc::new(Barrier::new(2));
        let worker = {
            let handle = handle.clone();
            let ready = ready.clone();
            let resume = resume.clone();
            std::thread::spawn(move || {
                ready.wait();
                resume.wait();
                handle.arm_session(admission)
            })
        };
        ready.wait();
        assert_eq!(cancel(&handle).phase, ProbePhase::Stopped);
        resume.wait();
        assert_eq!(worker.join().unwrap(), Err(RefusalReason::StaleContext));
        assert!(!handle.shared.gate.is_active());
    }

    #[test]
    fn construction_and_stop_do_not_start_native_observation() {
        let handle = ProbeHandle::new();
        assert_eq!(handle.status().phase, ProbePhase::Stopped);
        assert!(!handle.shared.gate.is_active());
        assert_eq!(handle.stop().phase, ProbePhase::Stopped);
        assert_eq!(handle.status().counters.submitted, 0);
    }

    #[test]
    fn content_invalidation_prevents_resuming_a_paused_old_library_without_native_work() {
        let handle = ProbeHandle::new();
        let old = Session {
            policy: TargetPolicy::NotepadEditor,
            #[cfg(target_os = "windows")]
            rules: Arc::new(RuleSet::fixtures()),
            profile: None,
            library_epoch: 0,
        };
        handle.invalidate_library();
        assert_eq!(handle.start_session(old), Err(RefusalReason::StaleContext));
        assert!(!handle.shared.gate.is_active());
        assert!(handle.session.lock().unwrap().is_none());
    }

    #[test]
    fn exclusions_refuse_before_threads_or_hooks_are_created() {
        let handle = ProbeHandle::new();
        handle.exclude(ReferenceProfile::Notepad);
        let result = handle.start(StartOptions {
            profile: ReferenceProfile::Notepad,
            strategy: super::super::types::InsertionStrategy::Unicode,
        });
        assert_eq!(result, Err(RefusalReason::Excluded));
        assert!(!handle.shared.gate.is_active());
    }

    #[test]
    fn native_gate_closure_cannot_leave_status_running() {
        let handle = ProbeHandle::new();
        handle.shared.gate.arm().unwrap();
        handle.shared.status.lock().unwrap().phase = ProbePhase::Running;
        handle.shared.gate.close();
        assert_eq!(handle.status().phase, ProbePhase::Fault);
        assert_eq!(
            handle.status().refusal_reason,
            Some(RefusalReason::StaleContext)
        );
    }

    #[test]
    fn exclusion_cancels_start_before_selected_profile_is_published() {
        let handle = ProbeHandle::new();
        handle.shared.gate.arm().unwrap();
        handle.exclude(ReferenceProfile::Notepad);
        assert!(!handle.shared.gate.is_active());
    }

    #[test]
    fn zero_queued_events_do_not_increment_submitted_counter() {
        let handle = ProbeHandle::new();
        handle
            .shared
            .record(OperationResult::Uncertain { submitted: false });
        assert_eq!(handle.status().counters.submitted, 0);
        assert_eq!(handle.status().counters.uncertain, 1);
        assert_eq!(handle.status().phase, ProbePhase::Fault);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn clipboard_refuses_all_profiles_before_creating_a_native_job() {
        let handle = ProbeHandle::new();
        for profile in [
            ReferenceProfile::Notepad,
            ReferenceProfile::VsCode,
            ReferenceProfile::ChromeFixture,
        ] {
            assert_eq!(
                handle.start(StartOptions {
                    profile,
                    strategy: super::super::types::InsertionStrategy::Clipboard,
                }),
                Err(RefusalReason::ClipboardState)
            );
            assert!(!handle.shared.gate.is_active());
            assert!(handle.job.lock().unwrap().is_none());
            assert_eq!(handle.status(), ProbeStatus::default());
        }
    }
}
