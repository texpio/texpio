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
}
type UsageRecorder = Arc<dyn Fn(&str) + Send + Sync>;
pub struct ProductEngine {
    pub(crate) shared: Arc<Shared>,
    publication: Mutex<()>,
    #[cfg(target_os = "windows")]
    runtime: Mutex<Option<super::hooks::HookRuntime>>,
}
impl Default for ProductEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl ProductEngine {
    pub fn new() -> Self {
        Self {
            publication: Mutex::new(()),
            shared: Arc::new(Shared {
                generation: Arc::new(AtomicU64::new(0)),
                active: AtomicBool::new(false),
                busy: AtomicBool::new(false),
                status: Mutex::new(ProductStatus::default()),
                used: Mutex::new(None),
            }),
            #[cfg(target_os = "windows")]
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
    pub fn pause(&self) -> ProductStatus {
        let result = self.stop();
        if result.phase == ProductPhase::Stopped {
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
        self.shared.generation.load(Ordering::SeqCst)
    }
    fn admit(&self, ticket: u64) -> Result<(), ProductError> {
        let _publication = self
            .publication
            .lock()
            .map_err(|_| ProductError::Unavailable)?;
        if self.admission() != ticket {
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
        if !settings.validate() {
            return Err(ProductError::InvalidSettings);
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = rules;
            Err(ProductError::Unavailable)
        }
        #[cfg(target_os = "windows")]
        {
            let mut runtime = self.runtime.lock().map_err(|_| ProductError::Busy)?;
            if runtime.is_some() {
                return Err(ProductError::Busy);
            }
            self.admit(ticket)?;
            match super::hooks::HookRuntime::launch(self.shared.clone(), rules, settings) {
                Ok(worker) => {
                    *runtime = Some(worker);
                    let publication = self
                        .publication
                        .lock()
                        .map_err(|_| ProductError::Unavailable)?;
                    if !self.shared.active.load(Ordering::SeqCst) || self.admission() != ticket {
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
        {
            let _publication = self.publication.lock().unwrap_or_else(|p| p.into_inner());
            self.shared.active.store(false, Ordering::SeqCst);
            self.shared.generation.fetch_add(1, Ordering::SeqCst);
        }
        #[cfg(target_os = "windows")]
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
        #[cfg(not(target_os = "windows"))]
        let stopped = true;
        let mut status = self.shared.status.lock().unwrap_or_else(|p| p.into_inner());
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
        status.clone()
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
