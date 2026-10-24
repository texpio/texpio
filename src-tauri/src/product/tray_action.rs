use super::{
    effects::ProductError,
    service::{ProductEngine, ProductPhase, ProductStatus},
};
use std::sync::atomic::Ordering;

pub(crate) struct ToggleRequest {
    pub ticket: u64,
    enable: bool,
}
impl ToggleRequest {
    pub fn capture(engine: &ProductEngine) -> Self {
        Self {
            ticket: engine.admission(),
            enable: engine.status().phase != ProductPhase::Running
                && !engine.shared.busy.load(Ordering::Acquire),
        }
    }
    pub fn run(
        self,
        engine: &ProductEngine,
        start: impl FnOnce(u64) -> Result<ProductStatus, ProductError>,
    ) -> Result<ProductStatus, ProductError> {
        if engine.admission() != self.ticket {
            return Err(ProductError::Cancelled);
        }
        if self.enable {
            start(self.ticket)
        } else {
            Ok(engine.pause())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    #[test]
    fn queued_enable_is_cancelled_by_newer_stop_and_pause_keeps_its_intent() {
        let engine = ProductEngine::new();
        let request = ToggleRequest::capture(&engine);
        engine.stop();
        let started = Cell::new(false);
        let result = request.run(&engine, |_| {
            started.set(true);
            Ok(engine.status())
        });
        assert!(matches!(result, Err(ProductError::Cancelled)));
        assert!(!started.get());
        engine.shared.active.store(true, Ordering::SeqCst);
        engine.shared.status.lock().unwrap().phase = ProductPhase::Running;
        let pause = ToggleRequest::capture(&engine);
        engine.shared.active.store(false, Ordering::SeqCst);
        engine.shared.status.lock().unwrap().phase = ProductPhase::Stopped;
        pause
            .run(&engine, |_| {
                panic!("A queued Pause must never become Enable")
            })
            .unwrap();
        assert_eq!(engine.status().phase, ProductPhase::Paused);
    }
}
