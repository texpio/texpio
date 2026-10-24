use super::*;

const FOLDER: &str = "8ed403c0-aa02-4200-b627-709970ebc1b4";
const SNIPPET: &str = "63c1fa02-f8ce-4041-8392-9795cbf3d410";
const SECOND: &str = "0bc9689c-c1ec-4232-b96e-5cc2e638c0ac";

fn fixture(revision: u32) -> SaveLibrary {
    SaveLibrary {
        expected_revision: revision,
        folders: vec![Folder {
            id: FOLDER.into(),
            name: "Personal".into(),
        }],
        snippets: vec![Snippet {
            id: SNIPPET.into(),
            folder_id: Some(FOLDER.into()),
            title: "Unicode greeting".into(),
            trigger: ";greet".into(),
            body: "Hello\t世界 🌍\r\nनमस्ते".into(),
            enabled: true,
            deleted: false,
        }],
    }
}

fn envelope(input: &SaveLibrary) -> String {
    serde_json::to_string(&Library {
        schema_version: 1,
        revision: 900,
        folders: input.folders.clone(),
        snippets: input.snippets.clone(),
    })
    .unwrap()
}

fn error_code<T: std::fmt::Debug>(result: Result<T, LibraryError>, code: &str) {
    assert_eq!(result.unwrap_err().code, code);
}

#[test]
fn creates_synthetic_samples_only_on_first_initialization() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let loaded = store.load().unwrap();
    assert_eq!(loaded.schema_version, 1);
    assert_eq!(loaded.revision, 0);
    assert_eq!(
        loaded
            .snippets
            .iter()
            .map(|s| s.trigger.as_str())
            .collect::<Vec<_>>(),
        [";hello", ";addr"]
    );
    store
        .save(SaveLibrary {
            expected_revision: 0,
            folders: vec![],
            snippets: vec![],
        })
        .unwrap();
    drop(store);
    let restarted = LibraryStore::new(root.path().into()).load().unwrap();
    assert!(restarted.snippets.is_empty());
    assert!(restarted.folders.is_empty());
    assert_eq!(restarted.revision, 1);
}

#[test]
fn committed_unicode_content_survives_restart() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let saved = store.save(fixture(0)).unwrap();
    assert_eq!(saved.revision, 1);
    drop(store);
    let loaded = LibraryStore::new(root.path().into()).load().unwrap();
    assert_eq!(loaded, saved);
    assert_eq!(loaded.snippets[0].body, "Hello\t世界 🌍\r\nनमस्ते");
}

#[test]
fn stale_revision_cannot_replace_acknowledged_content() {
    let root = tempfile::tempdir().unwrap();
    let first = LibraryStore::new(root.path().into());
    let second = LibraryStore::new(root.path().into());
    let saved = first.save(fixture(0)).unwrap();
    let mut stale = fixture(0);
    stale.snippets[0].body = "stale replacement".into();
    error_code(second.save(stale), "conflict");
    assert_eq!(first.load().unwrap(), saved);
}

#[test]
fn active_trigger_collisions_refuse_but_disabled_and_trash_duplicates_survive() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    let mut duplicate = input.snippets[0].clone();
    duplicate.id = SECOND.into();
    input.snippets.push(duplicate);
    error_code(store.save(input.clone()), "conflict");
    input.snippets[1].enabled = false;
    let disabled = store.save(input.clone()).unwrap();
    assert!(!disabled.snippets[1].enabled);
    input.expected_revision = 1;
    input.snippets[1].enabled = true;
    input.snippets[1].deleted = true;
    assert!(store.save(input).unwrap().snippets[1].deleted);
}

#[test]
fn invalid_ids_references_triggers_and_unsafe_controls_are_rejected_without_writes() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let initial = store.load().unwrap();
    for field in ["id", "folder", "trigger", "body", "title", "name"] {
        let mut input = fixture(0);
        match field {
            "id" => input.snippets[0].id = "not-a-uuid".into(),
            "folder" => input.snippets[0].folder_id = Some(SECOND.into()),
            "trigger" => input.snippets[0].trigger = ";Wrong1".into(),
            "body" => input.snippets[0].body = "text\u{8}hidden".into(),
            "title" => input.snippets[0].title = " ".into(),
            "name" => input.folders[0].name = "".into(),
            _ => unreachable!(),
        }
        error_code(store.save(input), "invalidData");
    }
    for control in ['\0', '\u{1b}', '\u{7f}', '\u{85}'] {
        let mut input = fixture(0);
        input.snippets[0].body = format!("before{control}after");
        error_code(store.save(input), "invalidData");
    }
    let mut empty = fixture(0);
    empty.snippets[0].body.clear();
    error_code(store.save(empty), "invalidData");
    assert_eq!(store.load().unwrap(), initial);
}

#[test]
fn unicode_scalar_limits_are_enforced_without_counting_utf8_bytes() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut valid = fixture(0);
    valid.snippets[0].body = "🌍".repeat(16_384);
    valid.snippets[0].title = "界".repeat(120);
    store.save(valid.clone()).unwrap();
    valid.expected_revision = 1;
    valid.snippets[0].body.push('a');
    error_code(store.save(valid), "tooLarge");
    let mut long_title = fixture(1);
    long_title.snippets[0].title = "界".repeat(121);
    error_code(store.save(long_title), "tooLarge");
}

#[test]
fn previews_additive_import_without_writes_and_skips_identical_records() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let before = store.load().unwrap();
    let json = envelope(&fixture(0));
    assert_eq!(
        store.preview_import(&json).unwrap(),
        ImportPreview {
            folders: 1,
            snippets: 1,
            skipped: 0
        }
    );
    assert_eq!(store.load().unwrap(), before);
    let imported = store.import_json(&json, 0).unwrap();
    assert_eq!(imported.revision, 1);
    assert_eq!(imported.snippets.len(), 3);
    assert_eq!(
        store.preview_import(&json).unwrap(),
        ImportPreview {
            folders: 0,
            snippets: 0,
            skipped: 2
        }
    );
    assert_eq!(store.import_json(&json, 1).unwrap(), imported);
}

#[test]
fn import_conflicts_and_stale_revisions_rollback_all_additions() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let saved = store.save(fixture(0)).unwrap();
    let mut incoming = fixture(0);
    incoming.folders[0].id = SECOND.into();
    incoming.snippets[0].folder_id = Some(SECOND.into());
    incoming.snippets[0].body = "different content at same ID".into();
    let json = envelope(&incoming);
    error_code(store.preview_import(&json), "conflict");
    error_code(store.import_json(&json, 1), "conflict");
    incoming.snippets[0].id = "ded5534a-46fb-41da-a3b4-2041c70d34bd".into();
    error_code(store.import_json(&envelope(&incoming), 1), "conflict");
    incoming.snippets[0].trigger = ";newtrigger".into();
    error_code(store.import_json(&envelope(&incoming), 0), "conflict");
    assert_eq!(store.load().unwrap(), saved);
}

#[test]
fn exported_json_round_trips_preserving_ids_and_ignoring_imported_revision() {
    let source_root = tempfile::tempdir().unwrap();
    let source = LibraryStore::new(source_root.path().into());
    let saved = source.save(fixture(0)).unwrap();
    let target_root = tempfile::tempdir().unwrap();
    let target = LibraryStore::new(target_root.path().into());
    target
        .save(SaveLibrary {
            expected_revision: 0,
            folders: vec![],
            snippets: vec![],
        })
        .unwrap();
    let restored = target
        .import_json(&source.export_json().unwrap(), 1)
        .unwrap();
    assert_eq!(restored.revision, 2);
    assert_eq!(restored.snippets, saved.snippets);
    assert_eq!(restored.folders, saved.folders);
}

#[test]
fn version_unknown_fields_and_size_are_bounded_before_import() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut json: serde_json::Value = serde_json::from_str(&envelope(&fixture(0))).unwrap();
    json["schemaVersion"] = 2.into();
    error_code(
        store.preview_import(&json.to_string()),
        "unsupportedVersion",
    );
    json["schemaVersion"] = 1.into();
    json["unexpected"] = true.into();
    error_code(store.preview_import(&json.to_string()), "invalidData");
    error_code(
        store.preview_import(&" ".repeat(8 * 1024 * 1024 + 1)),
        "tooLarge",
    );
}

#[test]
fn destructive_save_and_import_keep_recoverable_previous_snapshots() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let original = store.load().unwrap();
    let saved = store.save(fixture(0)).unwrap();
    let backup: Library = serde_json::from_slice(
        &std::fs::read(root.path().join("backups/library-r0.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(backup, original);
    let mut addition = fixture(0);
    addition.snippets[0].id = SECOND.into();
    addition.snippets[0].trigger = ";second".into();
    store.import_json(&envelope(&addition), 1).unwrap();
    let backup: Library = serde_json::from_slice(
        &std::fs::read(root.path().join("backups/library-r1.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(backup, saved);
}

#[test]
fn failed_backup_refuses_changes_and_preserves_known_good_database() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let original = store.load().unwrap();
    std::fs::create_dir_all(root.path().join("backups/library-r0.json")).unwrap();
    error_code(store.save(fixture(0)), "storage");
    assert_eq!(store.load().unwrap(), original);
}

#[test]
fn duplicate_ids_and_cross_kind_collisions_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    input.snippets.push(input.snippets[0].clone());
    error_code(store.save(input), "invalidData");
    let mut input = fixture(0);
    input.snippets[0].id = FOLDER.into();
    error_code(store.save(input), "invalidData");
}

#[test]
fn concurrent_handles_cannot_both_commit_the_same_revision() {
    let root = tempfile::tempdir().unwrap();
    LibraryStore::new(root.path().into()).load().unwrap();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let path = root.path().to_owned();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                LibraryStore::new(path).save(fixture(0))
            })
        })
        .collect();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .next()
            .unwrap()
            .code,
        "conflict"
    );
    assert_eq!(
        LibraryStore::new(root.path().into())
            .load()
            .unwrap()
            .revision,
        1
    );
}

#[test]
fn oversized_record_counts_and_total_json_refuse_without_changing_library() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let original = store.load().unwrap();
    let mut many_folders = fixture(0);
    many_folders.folders = (0..1001)
        .map(|index| Folder {
            id: format!("10000000-0000-4000-8000-{index:012x}"),
            name: "Folder".into(),
        })
        .collect();
    many_folders.snippets.clear();
    error_code(store.save(many_folders), "tooLarge");
    let mut many_snippets = fixture(0);
    many_snippets.snippets = (0..10001)
        .map(|index| Snippet {
            id: format!("20000000-0000-4000-8000-{index:012x}"),
            folder_id: None,
            title: "Snippet".into(),
            trigger: ";duplicate".into(),
            body: "Value".into(),
            enabled: false,
            deleted: false,
        })
        .collect();
    error_code(store.save(many_snippets), "tooLarge");
    let mut large_json = fixture(0);
    large_json.snippets = (0..140)
        .map(|index| Snippet {
            id: format!("30000000-0000-4000-8000-{index:012x}"),
            folder_id: None,
            title: "Large Unicode snippet".into(),
            trigger: ";duplicate".into(),
            body: "🌍".repeat(16384),
            enabled: false,
            deleted: false,
        })
        .collect();
    error_code(store.save(large_json), "tooLarge");
    assert_eq!(store.load().unwrap(), original);
}

#[test]
fn merged_import_json_limit_is_checked_even_when_each_library_is_valid() {
    let source_root = tempfile::tempdir().unwrap();
    let source = LibraryStore::new(source_root.path().into());
    let target_root = tempfile::tempdir().unwrap();
    let target = LibraryStore::new(target_root.path().into());
    for (store, prefix) in [(&source, 4), (&target, 5)] {
        let mut input = fixture(0);
        input.snippets = (0..70)
            .map(|index| Snippet {
                id: format!("{prefix}0000000-0000-4000-8000-{index:012x}"),
                folder_id: None,
                title: "Large Unicode snippet".into(),
                trigger: ";duplicate".into(),
                body: "🌍".repeat(16384),
                enabled: false,
                deleted: false,
            })
            .collect();
        store.save(input).unwrap();
    }
    let previous = target.load().unwrap();
    let incoming = source.export_json().unwrap();
    assert!(incoming.len() < 8 * 1024 * 1024);
    error_code(target.preview_import(&incoming), "tooLarge");
    error_code(target.import_json(&incoming, 1), "tooLarge");
    assert_eq!(target.load().unwrap(), previous);
}

#[test]
fn sqlite_abort_during_import_rolls_back_the_entire_transaction() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let previous = store.save(fixture(0)).unwrap();
    let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    connection.execute_batch(
        "CREATE TRIGGER refuse_new_snippet BEFORE INSERT ON snippets
        WHEN NEW.id='0bc9689c-c1ec-4232-b96e-5cc2e638c0ac' BEGIN SELECT RAISE(ABORT, 'test refusal'); END;"
    ).unwrap();
    drop(connection);
    let mut incoming = fixture(0);
    incoming.folders[0].id = "bf7f3650-a5d3-4164-81a6-9868d720f5e3".into();
    incoming.snippets[0].id = SECOND.into();
    incoming.snippets[0].folder_id = Some(incoming.folders[0].id.clone());
    incoming.snippets[0].trigger = ";addition".into();
    error_code(store.import_json(&envelope(&incoming), 1), "storage");
    assert_eq!(store.load().unwrap(), previous);
    let backup: Library = serde_json::from_slice(
        &std::fs::read(root.path().join("backups/library-r1.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(backup, previous);
}

#[test]
fn future_database_schema_refuses_without_reinitializing() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    store.load().unwrap();
    let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    connection.execute_batch("PRAGMA user_version=2;").unwrap();
    drop(connection);
    error_code(store.load(), "unsupportedVersion");
    let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM snippets", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        2
    );
}

#[test]
fn import_accepts_a_missing_revision_and_literal_tokens_without_evaluating_them() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    input.snippets[0].body = "{{date}} ${clipboard} plain literal".into();
    let mut json: serde_json::Value = serde_json::from_str(&envelope(&input)).unwrap();
    json.as_object_mut().unwrap().remove("revision");
    let imported = store.import_json(&json.to_string(), 0).unwrap();
    assert_eq!(imported.revision, 1);
    assert_eq!(
        imported.snippets.last().unwrap().body,
        "{{date}} ${clipboard} plain literal"
    );
}
