#[cfg(target_os = "windows")]
use super::windows::worker::WindowsJob;
use super::{
    control::{CloseOutcome, DispatchGate},
    runtime::{OperationResult, StartOptions},
    types::{ProbePhase, ProbeStatus, ReferenceProfile, RefusalReason},
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
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
    pub fn reason(&self, reason: Option<RefusalReason>) {
        let mut status = self.status.lock().unwrap_or_else(|p| p.into_inner());
        if status.phase == ProbePhase::Running {
            status.refusal_reason = reason;
        }
    }
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
    #[cfg(target_os = "windows")]
    job: Mutex<Option<WindowsJob>>,
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
            #[cfg(target_os = "windows")]
            job: Mutex::new(None),
        }
    }
    pub fn status(&self) -> ProbeStatus {
        self.shared.snapshot()
    }

    pub fn start(&self, options: StartOptions) -> Result<ProbeStatus, RefusalReason> {
        if self
            .excluded
            .lock()
            .map_err(|_| RefusalReason::OutcomeUncertain)?
            .contains(&options.profile)
        {
            return Err(RefusalReason::Excluded);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = options;
            Err(RefusalReason::UnsupportedPlatform)
        }
        #[cfg(target_os = "windows")]
        {
            if options.strategy != super::types::InsertionStrategy::Unicode {
                return Err(RefusalReason::UnsupportedProfile);
            }
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
            self.shared.gate.arm()?;
            self.shared.uncertain.store(false, Ordering::SeqCst);
            *self
                .shared
                .status
                .lock()
                .map_err(|_| RefusalReason::OutcomeUncertain)? = ProbeStatus {
                phase: ProbePhase::Running,
                profile: Some(options.profile),
                strategy: options.strategy,
                ..ProbeStatus::default()
            };
            let mut job = match WindowsJob::spawn(self.shared.clone(), options.profile) {
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
            if !self.shared.gate.is_active() {
                return Err(RefusalReason::Stopped);
            }
            Ok(self.status())
        }
    }

    pub fn stop(&self) -> ProbeStatus {
        let outcome = self.shared.gate.close();
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
            let stopped = self.stop();
            if stopped.phase == ProbePhase::Stopped {
                self.shared
                    .status
                    .lock()
                    .map_err(|_| RefusalReason::OutcomeUncertain)?
                    .phase = ProbePhase::Paused;
            }
            Ok(self.status())
        } else {
            let status = self.status();
            self.start(StartOptions {
                profile: status.profile.ok_or(RefusalReason::Stopped)?,
                strategy: status.strategy,
            })
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
    fn construction_and_stop_do_not_start_native_observation() {
        let handle = ProbeHandle::new();
        assert_eq!(handle.status().phase, ProbePhase::Stopped);
        assert!(!handle.shared.gate.is_active());
        assert_eq!(handle.stop().phase, ProbePhase::Stopped);
        assert_eq!(handle.status().counters.submitted, 0);
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
}
