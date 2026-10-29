use super::{effects::ProductError, matcher::ProductRule, settings::ProductSettings};
use serde::Serialize;
use specta::Type;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ProductPhase {
    #[default]
    Stopped,
    Running,
    Paused,
    Fault,
}
#[derive(Clone, Debug, Default, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProductStatus {
    pub phase: ProductPhase,
    pub submitted: u32,
    pub cancelled: u32,
    pub last_error: Option<ProductError>,
    pub clipboard_incomplete: bool,
}
pub(crate) struct Shared {
    pub generation: Arc<AtomicU64>,
    pub active: AtomicBool,
    pub busy: AtomicBool,
    pub status: Mutex<ProductStatus>,
    pub used: Mutex<Option<UsageRecorder>>,
    pub files: Mutex<Option<FileResolver>>,
    pub effects: Mutex<()>,
}
type UsageRecorder = Arc<dyn Fn(&str) + Send + Sync>;
type FileResolver =
    Arc<dyn Fn(&str) -> Result<super::payload::FileDelivery, ProductError> + Send + Sync>;
pub struct ProductEngine {
    pub(crate) shared: Arc<Shared>,
    publication: Mutex<u64>,
    runtime: Mutex<Option<super::platform::Runtime>>,
}
impl Default for ProductEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl ProductEngine {
    pub(crate) fn suspend_admitted(
        &self,
        admission: u64,
    ) -> Result<(Option<u64>, u64), ProductError> {
        let (status, ticket, running) = self
            .stop_checked(Some(admission))
            .ok_or(ProductError::Cancelled)?;
        let publication = self
            .publication
            .lock()
            .map_err(|_| ProductError::Unavailable)?;
        if *publication != ticket {
            return Err(ProductError::Cancelled);
        }
        if status.phase != ProductPhase::Stopped || self.shared.active.load(Ordering::SeqCst) {
            return Err(ProductError::Busy);
        }
        Ok((
            running.then_some(ticket),
            self.shared.generation.load(Ordering::SeqCst),
        ))
    }
    pub fn new() -> Self {
        Self {
            publication: Mutex::new(0),
            shared: Arc::new(Shared {
                generation: Arc::new(AtomicU64::new(0)),
                active: AtomicBool::new(false),
                busy: AtomicBool::new(false),
                status: Mutex::new(ProductStatus::default()),
                used: Mutex::new(None),
                files: Mutex::new(None),
                effects: Mutex::new(()),
            }),
            runtime: Mutex::new(None),
        }
    }
    pub fn status(&self) -> ProductStatus {
        self.shared
            .status
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
    }
    pub(crate) fn record_with(&self, recorder: impl Fn(&str) + Send + Sync + 'static) {
        *self.shared.used.lock().unwrap_or_else(|p| p.into_inner()) = Some(Arc::new(recorder));
    }
    pub(crate) fn files_with(
        &self,
        resolver: impl Fn(&str) -> Result<super::payload::FileDelivery, ProductError>
        + Send
        + Sync
        + 'static,
    ) {
        *self.shared.files.lock().unwrap_or_else(|p| p.into_inner()) = Some(Arc::new(resolver));
    }
    pub(crate) fn suspend_for_update(&self) -> Result<Option<u64>, ProductError> {
        let (status, ticket, running) = self.stop_inner();
        if status.phase != ProductPhase::Stopped {
            return Err(ProductError::Busy);
        }
        Ok(running.then_some(ticket))
    }
    pub fn pause(&self) -> ProductStatus {
        let (result, ticket, _) = self.stop_inner();
        let publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
        if result.phase == ProductPhase::Stopped
            && *publication == ticket
            && !self.shared.active.load(Ordering::SeqCst)
        {
            self.shared
                .status
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .phase = ProductPhase::Paused;
        }
        self.status()
    }
    pub fn start(
        &self,
        rules: Vec<ProductRule>,
        settings: ProductSettings,
    ) -> Result<ProductStatus, ProductError> {
        self.start_admitted(rules, settings, self.admission())
    }
    pub fn admission(&self) -> u64 {
        *self.publication.lock().unwrap_or_else(|p| p.into_inner())
    }
    pub(crate) fn record_desktop_result(
        &self,
        result: Result<(), ProductError>,
        resume_failure: Option<(u64, ProductError)>,
    ) {
        let operation_error = result.err();
        self.shared
            .status
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .last_error = operation_error;
        if operation_error.is_none()
            && let Some((ticket, error)) = resume_failure
        {
            self.record_resume_failure(ticket, error);
        }
    }
    pub(crate) fn record_resume_failure(&self, ticket: u64, error: ProductError) {
        if error == ProductError::Cancelled {
            return;
        }
        let publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
        if *publication != ticket || self.shared.active.load(Ordering::SeqCst) {
            return;
        }
        let mut status = self.shared.status.lock().unwrap_or_else(|p| p.into_inner());
        if status.phase == ProductPhase::Stopped {
            status.last_error = Some(error);
        }
    }
    fn admit(&self, ticket: u64) -> Result<(), ProductError> {
        let publication = self
            .publication
            .lock()
            .map_err(|_| ProductError::Unavailable)?;
        if *publication != ticket {
            return Err(ProductError::Cancelled);
        }
        self.shared.active.store(true, Ordering::SeqCst);
        Ok(())
    }
    pub(crate) fn start_admitted(
        &self,
        rules: Vec<ProductRule>,
        settings: ProductSettings,
        ticket: u64,
    ) -> Result<ProductStatus, ProductError> {
        if !settings.validate_for_platform(std::env::consts::OS) {
            return Err(ProductError::InvalidSettings);
        }
        {
            let mut runtime = self.runtime.lock().map_err(|_| ProductError::Busy)?;
            if runtime.is_some() {
                return Err(ProductError::Busy);
            }
            self.admit(ticket)?;
            match super::platform::Runtime::launch(self.shared.clone(), rules, settings) {
                Ok(worker) => {
                    *runtime = Some(worker);
                    let publication = self
                        .publication
                        .lock()
                        .map_err(|_| ProductError::Unavailable)?;
                    if !self.shared.active.load(Ordering::SeqCst) || *publication != ticket {
                        drop(publication);
                        if let Some(worker) = runtime.as_mut() {
                            worker.shutdown();
                        }
                        *runtime = None;
                        return Err(ProductError::Cancelled);
                    }
                    self.shared
                        .status
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .phase = ProductPhase::Running;
                    self.shared
                        .status
                        .lock()
                        .unwrap_or_else(|p| p.into_inner())
                        .last_error = None;
                    Ok(self.status())
                }
                Err(error) => {
                    self.shared.active.store(false, Ordering::SeqCst);
                    Err(error)
                }
            }
        }
    }
    pub fn stop(&self) -> ProductStatus {
        self.stop_inner().0
    }
    fn stop_inner(&self) -> (ProductStatus, u64, bool) {
        self.stop_checked(None)
            .unwrap_or_else(|| (self.status(), self.admission(), false))
    }
    fn stop_checked(&self, expected: Option<u64>) -> Option<(ProductStatus, u64, bool)> {
        let (ticket, running) = {
            let mut publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
            if expected.is_some_and(|expected| expected != *publication) {
                return None;
            }
            *publication = publication.wrapping_add(1);
            let running = self.shared.active.swap(false, Ordering::SeqCst);
            self.shared.generation.fetch_add(1, Ordering::SeqCst);
            (*publication, running)
        };
        let stopped = self
            .runtime
            .lock()
            .map(|mut runtime| {
                if let Some(worker) = runtime.as_mut()
                    && !worker.shutdown()
                {
                    return false;
                }
                *runtime = None;
                true
            })
            .unwrap_or(false);
        let publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
        let mut status = self.shared.status.lock().unwrap_or_else(|p| p.into_inner());
        if *publication != ticket || self.shared.active.load(Ordering::SeqCst) {
            return Some((status.clone(), ticket, running));
        }
        status.phase = if stopped {
            ProductPhase::Stopped
        } else {
            ProductPhase::Fault
        };
        status.last_error = if stopped {
            None
        } else {
            Some(ProductError::Busy)
        };
        Some((status.clone(), ticket, running))
    }
}
impl Drop for ProductEngine {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desktop_result_keeps_delivery_failure_and_reports_current_resume_failure() {
        let engine = ProductEngine::new();
        let (_, ticket, _) = engine.stop_inner();
        engine.record_desktop_result(Ok(()), Some((ticket, ProductError::Unavailable)));
        assert_eq!(engine.status().last_error, Some(ProductError::Unavailable));

        engine.record_desktop_result(
            Err(ProductError::InputBlocked),
            Some((ticket, ProductError::Unavailable)),
        );
        assert_eq!(engine.status().last_error, Some(ProductError::InputBlocked));

        engine.stop();
        engine.record_desktop_result(Ok(()), Some((ticket, ProductError::Unavailable)));
        assert_eq!(engine.status().last_error, None);
        let current = engine.admission();
        engine.record_desktop_result(Ok(()), Some((current, ProductError::Cancelled)));
        assert_eq!(engine.status().last_error, None);
    }
    #[test]
    fn resume_failure_is_reported_only_for_the_current_stopped_generation() {
        let engine = ProductEngine::new();
        let (_, ticket, _) = engine.stop_inner();
        engine.record_resume_failure(ticket, ProductError::Unavailable);
        assert_eq!(engine.status().last_error, Some(ProductError::Unavailable));

        let (_, newer_ticket, _) = engine.stop_inner();
        engine.record_resume_failure(ticket, ProductError::Busy);
        assert_eq!(engine.status().last_error, None);
        engine.record_resume_failure(newer_ticket, ProductError::Cancelled);
        assert_eq!(engine.status().last_error, None);
        engine.admit(newer_ticket).unwrap();
        engine.shared.status.lock().unwrap().phase = ProductPhase::Running;
        engine.record_resume_failure(newer_ticket, ProductError::Unavailable);
        assert_eq!(engine.status().last_error, None);
        assert_eq!(engine.status().phase, ProductPhase::Running);
    }
    #[test]
    fn delayed_clipboard_request_cannot_run_after_stop() {
        let engine = ProductEngine::new();
        let request = engine.admission();
        engine.stop();
        assert_eq!(
            engine.suspend_admitted(request),
            Err(ProductError::Cancelled)
        );
        let current = engine.admission();
        let (_, input) = engine.suspend_admitted(current).unwrap();
        engine.stop();
        assert_ne!(engine.shared.generation.load(Ordering::SeqCst), input);
    }
    #[test]
    fn edit_suspension_never_restarts_an_engine_that_was_already_stopped() {
        let engine = ProductEngine::new();
        assert_eq!(engine.suspend_for_update().unwrap(), None);
    }
    #[test]
    fn edit_suspension_closes_output_and_later_user_stop_cancels_its_resume() {
        let engine = ProductEngine::new();
        engine.shared.active.store(true, Ordering::SeqCst);
        let resume = engine.suspend_for_update().unwrap().unwrap();
        assert!(!engine.shared.active.load(Ordering::SeqCst));
        engine.stop();
        assert_eq!(engine.admit(resume), Err(ProductError::Cancelled));
    }
    #[test]
    fn stop_cancels_admission_before_any_native_hook_can_start() {
        let engine = ProductEngine::new();
        let ticket = engine.admission();
        engine.stop();
        assert_eq!(engine.admit(ticket), Err(ProductError::Cancelled));
        assert!(!engine.shared.active.load(Ordering::SeqCst));
    }
    #[test]
    fn construction_and_stop_have_no_native_side_effects_and_cancel_prior_tickets() {
        let engine = ProductEngine::new();
        let ticket = engine.shared.generation.load(Ordering::SeqCst);
        assert_eq!(engine.status().phase, ProductPhase::Stopped);
        assert_eq!(engine.stop().phase, ProductPhase::Stopped);
        assert_ne!(engine.shared.generation.load(Ordering::SeqCst), ticket);
        assert!(!engine.shared.active.load(Ordering::SeqCst));
    }
}
