pub use super::handle::ProbeHandle;
use super::types::{InsertionStrategy, ReferenceProfile};
use super::{
    control::{DispatchGate, GateSnapshot},
    types::{Candidate, RefusalReason},
};
use serde::Deserialize;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartOptions {
    pub profile: ReferenceProfile,
    pub strategy: InsertionStrategy,
}

pub struct ObservedCandidate<O> {
    pub candidate: Candidate,
    pub origin: O,
    pub ticket: GateSnapshot,
    pub published_at: Instant,
}

pub struct DispatchCounts {
    pub expected: u32,
    pub submitted: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OperationResult {
    Observed,
    Refused(RefusalReason),
    Uncertain { submitted: bool },
}

/// Construct the native backend on its owning worker; it need not be Send.
pub trait Backend {
    type Origin;
    type Edit;
    fn prepare(
        &mut self,
        request: &ObservedCandidate<Self::Origin>,
    ) -> Result<Self::Edit, RefusalReason>;
    fn revalidate(&mut self, edit: &Self::Edit) -> Result<(), RefusalReason>;
    fn dispatch(&mut self, edit: &Self::Edit) -> Result<DispatchCounts, RefusalReason>;
    fn verify(&mut self, edit: &Self::Edit, ticket: GateSnapshot) -> Result<bool, RefusalReason>;
}

pub struct RuntimeCore<B: Backend> {
    gate: Arc<DispatchGate>,
    backend: B,
}

impl<B: Backend> RuntimeCore<B> {
    pub fn new(gate: Arc<DispatchGate>, backend: B) -> Self {
        Self { gate, backend }
    }
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn process(&mut self, request: &ObservedCandidate<B::Origin>) -> OperationResult {
        let result = self.prepare_and_dispatch(request);
        if matches!(result, OperationResult::Uncertain { .. }) {
            self.gate.close();
        }
        result
    }

    fn prepare_and_dispatch(&mut self, request: &ObservedCandidate<B::Origin>) -> OperationResult {
        if let Err(reason) = self.gate.validate(request.ticket) {
            return OperationResult::Refused(reason);
        }
        if request.published_at.elapsed() >= Duration::from_millis(250) {
            return OperationResult::Refused(RefusalReason::ProviderTimeout);
        }
        let edit = match self.backend.prepare(request) {
            Ok(edit) => edit,
            Err(reason) => return OperationResult::Refused(reason),
        };
        if let Err(reason) = self.gate.validate(request.ticket) {
            return OperationResult::Refused(reason);
        }
        if let Err(reason) = self.backend.revalidate(&edit) {
            return OperationResult::Refused(reason);
        }
        if request.published_at.elapsed() >= Duration::from_millis(250) {
            return OperationResult::Refused(RefusalReason::ProviderTimeout);
        }
        let mut submitted = false;
        let counts = match self.gate.run(request.ticket, || {
            let result = self.backend.dispatch(&edit);
            if let Ok(counts) = &result {
                submitted = counts.submitted > 0;
            }
            result
        }) {
            Ok(counts) => counts,
            Err(RefusalReason::OutcomeUncertain) => {
                return OperationResult::Uncertain { submitted };
            }
            Err(reason) => return OperationResult::Refused(reason),
        };
        if counts.expected == 0 || counts.expected != counts.submitted {
            return OperationResult::Uncertain { submitted };
        }
        // A consumed ticket advances exactly once. Do not bless intervening physical
        // input by taking a fresh arbitrary snapshot after dispatch.
        let observed_ticket = GateSnapshot {
            context: request.ticket.context,
            input: request.ticket.input.wrapping_add(1),
        };
        if self.gate.validate(observed_ticket).is_err() {
            return OperationResult::Uncertain { submitted: true };
        }
        match self.backend.verify(&edit, observed_ticket) {
            Ok(true) if self.gate.validate(observed_ticket).is_ok() => OperationResult::Observed,
            _ => OperationResult::Uncertain { submitted: true },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{control::DispatchGate, types::Candidate};
    use std::sync::Arc;

    #[derive(Clone, Copy)]
    enum Scenario {
        Safe,
        StopDuringPrepare,
        ChangedControl,
        Partial,
        ZeroSubmitted,
        WrongOutput,
    }
    struct Fake {
        gate: Arc<DispatchGate>,
        scenario: Scenario,
        dispatches: usize,
    }
    impl Backend for Fake {
        type Origin = u32;
        type Edit = ();
        fn prepare(&mut self, request: &ObservedCandidate<u32>) -> Result<(), RefusalReason> {
            if request.origin != 17 {
                return Err(RefusalReason::StaleContext);
            }
            if matches!(self.scenario, Scenario::StopDuringPrepare) {
                self.gate.close();
            }
            Ok(())
        }
        fn revalidate(&mut self, _: &()) -> Result<(), RefusalReason> {
            if matches!(self.scenario, Scenario::ChangedControl) {
                Err(RefusalReason::StaleContext)
            } else {
                Ok(())
            }
        }
        fn dispatch(&mut self, _: &()) -> Result<DispatchCounts, RefusalReason> {
            self.dispatches += 1;
            Ok(DispatchCounts {
                expected: 10,
                submitted: if matches!(self.scenario, Scenario::ZeroSubmitted) {
                    0
                } else if matches!(self.scenario, Scenario::Partial) {
                    3
                } else {
                    10
                },
            })
        }
        fn verify(&mut self, _: &(), _: GateSnapshot) -> Result<bool, RefusalReason> {
            Ok(!matches!(self.scenario, Scenario::WrongOutput))
        }
    }
    fn setup(scenario: Scenario) -> (RuntimeCore<Fake>, ObservedCandidate<u32>) {
        let gate = Arc::new(DispatchGate::new());
        let ticket = gate.arm().unwrap();
        let core = RuntimeCore::new(
            gate.clone(),
            Fake {
                gate,
                scenario,
                dispatches: 0,
            },
        );
        let request = ObservedCandidate {
            candidate: Candidate {
                fixture_id: "hello".into(),
                trigger: ";hello".into(),
            },
            origin: 17,
            ticket,
            published_at: Instant::now(),
        };
        (core, request)
    }

    #[test]
    fn accepted_candidate_dispatches_and_observes_once() {
        let (mut core, request) = setup(Scenario::Safe);
        assert_eq!(core.process(&request), OperationResult::Observed);
        assert_eq!(
            core.process(&request),
            OperationResult::Refused(RefusalReason::StaleContext)
        );
        assert_eq!(core.backend.dispatches, 1);
    }
    #[test]
    fn original_target_staleness_and_stop_refuse_with_zero_dispatches() {
        for scenario in [Scenario::ChangedControl, Scenario::StopDuringPrepare] {
            let (mut core, request) = setup(scenario);
            assert!(matches!(
                core.process(&request),
                OperationResult::Refused(_)
            ));
            assert_eq!(core.backend.dispatches, 0);
        }
        let (mut core, mut request) = setup(Scenario::Safe);
        request.origin = 18;
        assert_eq!(
            core.process(&request),
            OperationResult::Refused(RefusalReason::StaleContext)
        );
        assert_eq!(core.backend.dispatches, 0);
    }
    #[test]
    fn input_after_publication_and_old_requests_never_dispatch() {
        let (mut core, mut request) = setup(Scenario::Safe);
        core.gate
            .observe(crate::engine::control::InputOrigin::Physical);
        assert_eq!(
            core.process(&request),
            OperationResult::Refused(RefusalReason::StaleContext)
        );
        request.ticket = core.gate.snapshot();
        request.published_at = Instant::now() - Duration::from_secs(1);
        assert_eq!(
            core.process(&request),
            OperationResult::Refused(RefusalReason::ProviderTimeout)
        );
        assert_eq!(core.backend.dispatches, 0);
    }
    #[test]
    fn partial_or_unobserved_dispatch_faults_without_repair_or_replay() {
        for scenario in [Scenario::Partial, Scenario::WrongOutput] {
            let (mut core, request) = setup(scenario);
            assert_eq!(
                core.process(&request),
                OperationResult::Uncertain { submitted: true }
            );
            assert!(!core.gate.is_active());
            assert_eq!(
                core.process(&request),
                OperationResult::Refused(RefusalReason::Stopped)
            );
            assert_eq!(core.backend.dispatches, 1);
        }
    }

    #[test]
    fn zero_queued_events_are_uncertain_but_not_submitted() {
        let (mut core, request) = setup(Scenario::ZeroSubmitted);
        assert_eq!(
            core.process(&request),
            OperationResult::Uncertain { submitted: false }
        );
        assert_eq!(core.backend.dispatches, 1);
        assert!(!core.gate.is_active());
    }
}
