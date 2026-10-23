use super::types::RefusalReason;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GateSnapshot {
    pub context: u64,
    pub input: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputOrigin {
    Physical,
    Own,
    UnexpectedInjected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseOutcome {
    Stopped,
    InFlightUncertain,
}

pub struct DispatchGate {
    active: AtomicBool,
    context: AtomicU64,
    input: AtomicU64,
    dispatch: Mutex<()>,
}

impl Default for DispatchGate {
    fn default() -> Self {
        Self {
            active: AtomicBool::new(false),
            context: AtomicU64::new(0),
            input: AtomicU64::new(0),
            dispatch: Mutex::new(()),
        }
    }
}

impl DispatchGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> GateSnapshot {
        GateSnapshot {
            context: self.context.load(Ordering::SeqCst),
            input: self.input.load(Ordering::SeqCst),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn arm(&self) -> Result<GateSnapshot, RefusalReason> {
        let _permit = self.dispatch.try_lock().map_err(|_| RefusalReason::Busy)?;
        if self.is_active() {
            return Err(RefusalReason::Busy);
        }
        self.invalidate_context();
        self.active.store(true, Ordering::SeqCst);
        Ok(self.snapshot())
    }

    pub fn invalidate_context(&self) {
        self.context.fetch_add(1, Ordering::SeqCst);
        self.input.fetch_add(1, Ordering::SeqCst);
    }

    pub fn observe(&self, origin: InputOrigin) {
        match origin {
            InputOrigin::Own => {}
            InputOrigin::Physical => {
                self.input.fetch_add(1, Ordering::SeqCst);
            }
            InputOrigin::UnexpectedInjected => self.invalidate_context(),
        }
    }

    pub fn close(&self) -> CloseOutcome {
        // Close first. Never wait for preparation, a provider RPC, or input dispatch.
        self.active.store(false, Ordering::SeqCst);
        self.invalidate_context();
        match self.dispatch.try_lock() {
            Ok(_) => CloseOutcome::Stopped,
            Err(_) => CloseOutcome::InFlightUncertain,
        }
    }

    pub fn validate(&self, ticket: GateSnapshot) -> Result<(), RefusalReason> {
        if !self.is_active() {
            return Err(RefusalReason::Stopped);
        }
        if ticket != self.snapshot() {
            return Err(RefusalReason::StaleContext);
        }
        Ok(())
    }

    /// Preparation must happen outside this gate. The closure is the final bounded
    /// native stamp check plus one SendInput batch, never a blocking UIA operation.
    pub fn run<T>(
        &self,
        ticket: GateSnapshot,
        dispatch: impl FnOnce() -> Result<T, RefusalReason>,
    ) -> Result<T, RefusalReason> {
        let _permit = self.dispatch.try_lock().map_err(|_| RefusalReason::Busy)?;
        self.validate(ticket)?;
        let result = dispatch();
        // Stop/input arriving during an admitted batch makes its outcome uncertain.
        let unchanged = self.validate(ticket).is_ok();
        self.input.fetch_add(1, Ordering::SeqCst);
        if !unchanged {
            return Err(RefusalReason::OutcomeUncertain);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn stopped_and_stale_work_cannot_dispatch() {
        let gate = DispatchGate::new();
        let calls = AtomicUsize::new(0);
        assert_eq!(
            gate.run(gate.snapshot(), || {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }),
            Err(RefusalReason::Stopped)
        );
        let old = gate.arm().unwrap();
        gate.observe(InputOrigin::Physical);
        assert_eq!(
            gate.run(old, || {
                calls.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }),
            Err(RefusalReason::StaleContext)
        );
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn a_candidate_is_consumed_once() {
        let gate = DispatchGate::new();
        let ticket = gate.arm().unwrap();
        let mut calls = 0;
        assert_eq!(
            gate.run(ticket, || {
                calls += 1;
                Ok(())
            }),
            Ok(())
        );
        assert_eq!(
            gate.run(ticket, || {
                calls += 1;
                Ok(())
            }),
            Err(RefusalReason::StaleContext)
        );
        assert_eq!(calls, 1);
    }

    #[test]
    fn pause_or_stop_during_preparation_prevents_late_dispatch() {
        let gate = Arc::new(DispatchGate::new());
        let ticket = gate.arm().unwrap();
        let prepared = Arc::new(Barrier::new(2));
        let released = Arc::new(Barrier::new(2));
        let calls = Arc::new(AtomicUsize::new(0));
        let worker = {
            let gate = gate.clone();
            let prepared = prepared.clone();
            let released = released.clone();
            let calls = calls.clone();
            std::thread::spawn(move || {
                prepared.wait();
                released.wait();
                gate.run(ticket, || {
                    calls.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                })
            })
        };
        prepared.wait();
        assert_eq!(gate.close(), CloseOutcome::Stopped);
        released.wait();
        assert_eq!(worker.join().unwrap(), Err(RefusalReason::Stopped));
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn stop_acknowledges_an_inflight_operation_without_waiting() {
        let gate = Arc::new(DispatchGate::new());
        let ticket = gate.arm().unwrap();
        let entered = Arc::new(Barrier::new(2));
        let released = Arc::new(Barrier::new(2));
        let worker = {
            let gate = gate.clone();
            let entered = entered.clone();
            let released = released.clone();
            std::thread::spawn(move || {
                gate.run(ticket, || {
                    entered.wait();
                    released.wait();
                    Ok(())
                })
            })
        };
        entered.wait();
        assert_eq!(gate.close(), CloseOutcome::InFlightUncertain);
        assert_eq!(gate.arm(), Err(RefusalReason::Busy));
        released.wait();
        assert_eq!(worker.join().unwrap(), Err(RefusalReason::OutcomeUncertain));
        assert_eq!(gate.run(ticket, || Ok(())), Err(RefusalReason::Stopped));
    }

    #[test]
    fn own_generated_events_do_not_cancel_but_foreign_input_does() {
        let gate = DispatchGate::new();
        let ticket = gate.arm().unwrap();
        gate.observe(InputOrigin::Own);
        assert_eq!(gate.snapshot(), ticket);
        gate.observe(InputOrigin::UnexpectedInjected);
        assert_ne!(gate.snapshot().context, ticket.context);
        assert_eq!(
            gate.run(ticket, || Ok(())),
            Err(RefusalReason::StaleContext)
        );
    }

    #[test]
    fn overflow_and_restart_invalidate_old_generations() {
        let gate = DispatchGate::new();
        let old = gate.arm().unwrap();
        gate.invalidate_context();
        assert_eq!(gate.run(old, || Ok(())), Err(RefusalReason::StaleContext));
        gate.close();
        let new = gate.arm().unwrap();
        assert_ne!(old, new);
        assert_eq!(gate.run(old, || Ok(())), Err(RefusalReason::StaleContext));
    }
}
