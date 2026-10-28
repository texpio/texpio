//! Content operations and native-session publication share one serialized boundary.
#[path = "library_backups.rs"]
pub(crate) mod backups;
#[path = "library_files.rs"]
pub(crate) mod files;
#[path = "library_location.rs"]
pub(crate) mod location;
use crate::{
    engine::{
        rules::{Rule, RuleSet},
        runtime::ProbeHandle,
        types::{ProbePhase, ProbeStatus, RefusalReason},
    },
    library::{ImportPreview, Library, LibraryError, SaveLibrary},
    probe_commands::ProbeError,
};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, State};

pub struct LibraryService {
    store: crate::library::location::LibraryRouter,
    control: Mutex<()>,
    watching: std::sync::atomic::AtomicBool,
    published_revision: std::sync::atomic::AtomicU32,
    staged_backup: Mutex<Option<backups::StagedBackup>>,
}

impl LibraryService {
    pub(crate) fn coordinated<T>(
        &self,
        operation: impl FnOnce() -> T,
    ) -> Result<T, crate::product::effects::ProductError> {
        let _control = self
            .control
            .lock()
            .map_err(|_| crate::product::effects::ProductError::Busy)?;
        Ok(operation())
    }
    pub(crate) fn update_product<T>(
        &self,
        suspend: impl FnOnce() -> Result<Option<u64>, crate::product::effects::ProductError>,
        operation: impl FnOnce() -> T,
        resume: impl FnOnce(u64),
    ) -> Result<T, crate::product::effects::ProductError> {
        self.coordinated(|| {
            let ticket = suspend()?;
            let result = operation();
            if let Some(ticket) = ticket {
                resume(ticket);
            }
            Ok(result)
        })?
    }
    pub(crate) fn start_product_on_launch(
        &self,
        engine: &crate::product::service::ProductEngine,
        probe: &ProbeHandle,
        settings: &crate::product::store::SettingsStore,
    ) -> Result<crate::product::service::ProductStatus, crate::product::effects::ProductError> {
        self.coordinated(|| {
            // Startup is a single initial intent, never a retry after a user action.
            if engine.admission() != 0
                || engine.status().phase != crate::product::service::ProductPhase::Stopped
            {
                return Ok(engine.status());
            }
            if probe.stop().phase != ProbePhase::Stopped {
                return Err(crate::product::effects::ProductError::Busy);
            }
            self.resume_product_locked(engine, settings, 0)
        })?
    }
    pub(crate) fn start_product_admitted(
        &self,
        engine: &crate::product::service::ProductEngine,
        probe: &ProbeHandle,
        settings: &crate::product::store::SettingsStore,
        ticket: u64,
    ) -> Result<crate::product::service::ProductStatus, crate::product::effects::ProductError> {
        self.coordinated(|| {
            if engine.admission() != ticket {
                return Err(crate::product::effects::ProductError::Cancelled);
            }
            if probe.stop().phase != ProbePhase::Stopped {
                return Err(crate::product::effects::ProductError::Busy);
            }
            self.resume_product_locked(engine, settings, ticket)
        })?
    }
    // The caller holds the coordinator through suspension, mutation and publication.
    pub(crate) fn resume_product_locked(
        &self,
        engine: &crate::product::service::ProductEngine,
        settings: &crate::product::store::SettingsStore,
        ticket: u64,
    ) -> Result<crate::product::service::ProductStatus, crate::product::effects::ProductError> {
        let settings = settings.load()?;
        let library = self
            .store
            .load()
            .map_err(|_| crate::product::effects::ProductError::Unavailable)?;
        let rules = product_rules(&library)?;
        let usage = self
            .store
            .snapshot()
            .map_err(|_| crate::product::effects::ProductError::Unavailable)?;
        engine.record_with(move |id| {
            let usage = usage.clone();
            let id = id.to_owned();
            tauri::async_runtime::spawn_blocking(move || {
                let _ = usage.record_usage(&id);
            });
        });
        let files = self
            .store
            .snapshot()
            .map_err(|_| crate::product::effects::ProductError::Unavailable)?;
        engine.files_with(move |id| {
            let options = files
                .file_settings(id)
                .map_err(|_| crate::product::effects::ProductError::AttachmentMissing)?;
            let paths = files
                .delivery_files(id)
                .map_err(|_| crate::product::effects::ProductError::AttachmentMissing)?;
            Ok(crate::product::payload::FileDelivery {
                files: paths,
                first: options.attachments_first,
                strict: options.strict_order,
            })
        });
        let result = engine.start_admitted(rules, settings, ticket)?;
        self.published_revision
            .store(library.revision, std::sync::atomic::Ordering::Release);
        Ok(result)
    }
    pub(crate) fn resume_product_after_update(
        &self,
        engine: &crate::product::service::ProductEngine,
        settings: &crate::product::store::SettingsStore,
        ticket: u64,
    ) {
        if let Err(error) = self.resume_product_locked(engine, settings, ticket) {
            engine.record_resume_failure(ticket, error);
        }
    }
    pub(crate) fn start_probe_admitted(
        &self,
        product: &crate::product::service::ProductEngine,
        probe: &ProbeHandle,
        admission: u64,
        operation: impl FnOnce(&ProbeHandle) -> Result<ProbeStatus, RefusalReason>,
    ) -> Result<ProbeStatus, RefusalReason> {
        self.coordinated(|| {
            if product.admission() != admission {
                return Err(RefusalReason::StaleContext);
            }
            if product.stop().phase != crate::product::service::ProductPhase::Stopped {
                return Err(RefusalReason::Busy);
            }
            let ticket = admission.wrapping_add(1);
            if product.admission() != ticket {
                return Err(RefusalReason::Stopped);
            }
            let result = operation(probe);
            if product.admission() != ticket {
                probe.stop();
                return Err(RefusalReason::Stopped);
            }
            result
        })
        .map_err(|_| RefusalReason::Busy)?
    }
    pub fn new(root: PathBuf) -> Self {
        Self {
            store: crate::library::location::LibraryRouter::new(root),
            control: Mutex::new(()),
            watching: std::sync::atomic::AtomicBool::new(true),
            published_revision: std::sync::atomic::AtomicU32::new(u32::MAX),
            staged_backup: Mutex::new(None),
        }
    }

    #[cfg(test)]
    fn save(&self, handle: &ProbeHandle, input: SaveLibrary) -> Result<Library, LibraryError> {
        let _control = self.control.lock().map_err(|_| storage_error())?;
        stopped(handle)?;
        self.store.save(input)
    }

    #[cfg(test)]
    fn import(
        &self,
        handle: &ProbeHandle,
        json: &str,
        revision: u32,
    ) -> Result<Library, LibraryError> {
        let _control = self.control.lock().map_err(|_| storage_error())?;
        stopped(handle)?;
        self.store.import_json(json, revision)
    }

    fn start_locked(&self, handle: &ProbeHandle) -> Result<ProbeStatus, RefusalReason> {
        let library = self
            .store
            .load()
            .map_err(|_| RefusalReason::ContextUnavailable)?;
        handle.start_library(Arc::new(rule_snapshot(&library)?))
    }
}

#[tauri::command]
#[specta::specta]
pub async fn library_metadata(
    service: State<'_, Arc<LibraryService>>,
) -> Result<Vec<crate::library::SnippetInfo>, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.metadata()).await
}
#[tauri::command]
#[specta::specta]
pub async fn library_set_favorite(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    id: String,
    revision: u32,
    favorite: bool,
) -> Result<(), LibraryError> {
    let service = service.inner().clone();
    blocking(move || {
        service
            .store
            .with_revision(revision, |store| store.set_favorite(&id, favorite))
    })
    .await?;
    let _ = app.emit("library-metadata-changed", ());
    Ok(())
}

fn storage_error() -> LibraryError {
    LibraryError {
        code: "storage".into(),
    }
}

fn stopped(handle: &ProbeHandle) -> Result<(), LibraryError> {
    if handle.invalidate_library().phase == ProbePhase::Stopped {
        Ok(())
    } else {
        Err(storage_error())
    }
}

fn product_rules(
    library: &Library,
) -> Result<Vec<crate::product::matcher::ProductRule>, crate::product::effects::ProductError> {
    use crate::product::{effects::ProductError, matcher::ProductRule};
    library
        .snippets
        .iter()
        .filter(|snippet| snippet.enabled && !snippet.deleted)
        .map(|snippet| {
            let rich_html = match &snippet.rich_body {
                None => None,
                Some(document) => {
                    let rendered = crate::rich_text::render(document)
                        .map_err(|_| ProductError::Unavailable)?;
                    if rendered.plain != snippet.body {
                        return Err(ProductError::Unavailable);
                    }
                    Some(rendered.html.into())
                }
            };
            Ok(ProductRule {
                id: snippet.id.clone(),
                trigger: snippet.trigger.clone(),
                body: snippet.body.clone().into(),
                rich_html,
            })
        })
        .collect()
}

fn rule_snapshot(library: &Library) -> Result<RuleSet, RefusalReason> {
    RuleSet::new(
        library.revision,
        library
            .snippets
            .iter()
            .filter(|item| item.enabled && !item.deleted)
            .map(|item| Rule {
                id: item.id.clone(),
                trigger: item.trigger.clone(),
                body: item.body.clone(),
            })
            .collect(),
    )
}

async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, LibraryError> + Send + 'static,
) -> Result<T, LibraryError> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| storage_error())?
}

#[tauri::command]
#[specta::specta]
pub async fn library_load(
    service: State<'_, Arc<LibraryService>>,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.load()).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_save(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
    settings: State<'_, Arc<crate::product::store::SettingsStore>>,
    input: SaveLibrary,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    let handle = handle.inner().clone();
    let product = product.inner().clone();
    let settings = settings.inner().clone();
    let result = blocking(move || {
        service
            .update_product(
                || product.suspend_for_update(),
                || {
                    stopped(&handle)?;
                    service.store.save(input)
                },
                |ticket| {
                    service.resume_product_after_update(&product, &settings, ticket);
                },
            )
            .map_err(|_| storage_error())?
    })
    .await;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    if result.is_ok() {
        let _ = app.emit("library-changed", ());
    }
    result
}

#[tauri::command]
#[specta::specta]
pub async fn library_export(
    service: State<'_, Arc<LibraryService>>,
) -> Result<String, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.export_json()).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_import_preview(
    service: State<'_, Arc<LibraryService>>,
    json: String,
) -> Result<ImportPreview, LibraryError> {
    let service = service.inner().clone();
    blocking(move || service.store.preview_import(&json)).await
}

#[tauri::command]
#[specta::specta]
pub async fn library_import(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
    settings: State<'_, Arc<crate::product::store::SettingsStore>>,
    json: String,
    expected_revision: u32,
) -> Result<Library, LibraryError> {
    let service = service.inner().clone();
    let handle = handle.inner().clone();
    let product = product.inner().clone();
    let settings = settings.inner().clone();
    let result = blocking(move || {
        service
            .update_product(
                || product.suspend_for_update(),
                || {
                    stopped(&handle)?;
                    service.store.import_json(&json, expected_revision)
                },
                |ticket| {
                    service.resume_product_after_update(&product, &settings, ticket);
                },
            )
            .map_err(|_| storage_error())?
    })
    .await;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    if result.is_ok() {
        let _ = app.emit("library-changed", ());
    }
    result
}

#[tauri::command]
#[specta::specta]
pub async fn library_start(
    app: AppHandle,
    service: State<'_, Arc<LibraryService>>,
    handle: State<'_, Arc<ProbeHandle>>,
    product: State<'_, Arc<crate::product::service::ProductEngine>>,
) -> Result<ProbeStatus, ProbeError> {
    let service = service.inner().clone();
    let handle = handle.inner().clone();
    let product = product.inner().clone();
    let admission = product.admission();
    let result = tauri::async_runtime::spawn_blocking(move || {
        service.start_probe_admitted(&product, &handle, admission, |handle| {
            service.start_locked(handle)
        })
    })
    .await
    .map_err(|_| ProbeError::from(RefusalReason::OutcomeUncertain))?;
    #[cfg(desktop)]
    crate::desktop::refresh_probe_tray(&app);
    result.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_rules_render_rich_content_and_keep_legacy_text_literal() {
        let directory = tempfile::tempdir().unwrap();
        let service = LibraryService::new(directory.path().into());
        let mut library = service.store.load().unwrap();
        library.snippets[0].body = "Café & tea".into();
        library.snippets[0].rich_body = Some(r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Café & tea","marks":[{"type":"bold"}]}]}]}}"#.into());
        library.snippets[1].body = "<b>literal</b>\r\n\t👋".into();
        let rules = product_rules(&library).unwrap();
        assert_eq!(&*rules[0].body, "Café & tea");
        assert_eq!(
            rules[0].rich_html.as_deref(),
            Some(
                "<div style=\"white-space:pre-wrap\"><p dir=\"auto\"><strong>Café &amp; tea</strong></p></div>"
            )
        );
        assert_eq!(&*rules[1].body, "<b>literal</b>\r\n\t👋");
        assert!(rules[1].rich_html.is_none());
        let mut history = crate::product::matcher::TypedHistory::new(
            rules.clone(),
            crate::product::matcher::Boundary::Anywhere,
            crate::product::matcher::Activation::Immediate,
        );
        let matched = library.snippets[0]
            .trigger
            .chars()
            .filter_map(|ch| history.push(ch))
            .next()
            .unwrap();
        assert_eq!(matched.rule.rich_html, rules[0].rich_html);
    }

    #[test]
    fn product_rules_reject_unsafe_or_mismatched_rich_documents() {
        let directory = tempfile::tempdir().unwrap();
        let service = LibraryService::new(directory.path().into());
        let mut library = service.store.load().unwrap();
        library.snippets[0].body = "visible".into();
        for document in [
            r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"other"}]}]}}"#,
            r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"visible","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}]}]}}"#,
            "{bad-json",
        ] {
            library.snippets[0].rich_body = Some(document.into());
            assert!(product_rules(&library).is_err());
        }
    }

    #[test]
    fn launch_never_replaces_a_running_session_or_rearms_after_pause_or_stop() {
        use crate::product::{
            service::{ProductEngine, ProductPhase},
            store::SettingsStore,
        };
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("expansion-settings.json");
        std::fs::write(&path, "{invalid").unwrap();
        let service = LibraryService::new(root.path().into());
        let settings = SettingsStore::new(root.path().into());
        let probe = ProbeHandle::new();
        let engine = ProductEngine::new();
        engine.shared.status.lock().unwrap().phase = ProductPhase::Running;
        assert_eq!(
            service
                .start_product_on_launch(&engine, &probe, &settings)
                .unwrap()
                .phase,
            ProductPhase::Running
        );
        engine.pause();
        assert_eq!(
            service
                .start_product_on_launch(&engine, &probe, &settings)
                .unwrap()
                .phase,
            ProductPhase::Paused
        );
        engine.stop();
        assert_eq!(
            service
                .start_product_on_launch(&engine, &probe, &settings)
                .unwrap()
                .phase,
            ProductPhase::Stopped
        );
        // A fresh process does attempt to read settings and fails closed before hooks.
        let fresh = ProductEngine::new();
        assert!(matches!(
            service.start_product_on_launch(&fresh, &probe, &settings),
            Err(crate::product::effects::ProductError::Unavailable)
        ));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "{invalid");
    }

    #[test]
    fn failed_resume_after_update_exposes_the_reason_without_starting_input() {
        use crate::product::{effects::ProductError, service::ProductEngine, store::SettingsStore};
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("expansion-settings.json"), "{invalid").unwrap();
        let service = LibraryService::new(root.path().into());
        let settings = SettingsStore::new(root.path().into());
        let engine = ProductEngine::new();
        service.resume_product_after_update(&engine, &settings, engine.admission());
        assert_eq!(engine.status().last_error, Some(ProductError::Unavailable));
        assert_eq!(
            engine.status().phase,
            crate::product::service::ProductPhase::Stopped
        );
    }

    #[test]
    fn snapshot_excludes_disabled_and_trashed_records() {
        let directory = tempfile::tempdir().unwrap();
        let service = LibraryService::new(directory.path().into());
        let mut library = service.store.load().unwrap();
        assert!(library.snippets.len() >= 2);
        let kept = library.snippets[0].id.clone();
        library.snippets[1].enabled = false;
        let mut trash = library.snippets[0].clone();
        trash.id = "00000000-0000-4000-8000-000000000099".into();
        trash.deleted = true;
        library.snippets.push(trash);
        let rules = rule_snapshot(&library).unwrap();
        assert_eq!(rules.entries().len(), 1);
        assert_eq!(rules.entries()[0].id, kept);
    }

    #[test]
    fn save_and_import_leave_observation_stopped_and_reject_stale_saves() {
        let directory = tempfile::tempdir().unwrap();
        let service = LibraryService::new(directory.path().into());
        let handle = ProbeHandle::new();
        let library = service.store.load().unwrap();
        let input = SaveLibrary {
            expected_revision: library.revision,
            folders: library.folders,
            snippets: library.snippets,
        };
        let mut changed = input.clone();
        changed.snippets[0].body = "Changed local body".into();
        let saved = service.save(&handle, changed).unwrap();
        assert_eq!(handle.status().phase, ProbePhase::Stopped);
        assert_eq!(service.save(&handle, input).unwrap_err().code, "conflict");
        let json = service.store.export_json().unwrap();
        assert_eq!(
            service.import(&handle, &json, saved.revision).unwrap(),
            saved
        );
        assert_eq!(handle.status().phase, ProbePhase::Stopped);
    }
}
