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
            rich_body: None,
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
            "trigger" => input.snippets[0].trigger = "has space".into(),
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
    empty.snippets[0].trigger.clear();
    error_code(store.save(empty), "invalidData");
    assert_eq!(store.load().unwrap(), initial);
}

#[test]
fn unicode_scalar_limits_are_enforced_without_counting_utf8_bytes() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut valid = fixture(0);
    valid.snippets[0].body = "🌍".repeat(MAX_BODY_SCALARS);
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
    json["schemaVersion"] = 3.into();
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
fn interchange_versions_preserve_plain_v1_and_validate_rich_v2() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    let saved = store.save(input.clone()).unwrap();
    assert_eq!(
        store.export_json().unwrap(),
        serde_json::to_string(&saved).unwrap()
    );

    input.expected_revision = 1;
    input.snippets[0].body = "Hello".into();
    input.snippets[0].rich_body = Some(r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello","marks":[{"type":"bold"}]}]}]}}"#.into());
    let saved = store.save(input).unwrap();
    assert_eq!(saved.schema_version, 1);
    let exported = store.export_json().unwrap();
    let mut document: serde_json::Value = serde_json::from_str(&exported).unwrap();
    assert_eq!(document["schemaVersion"], 2);
    let target_root = tempfile::tempdir().unwrap();
    let target = LibraryStore::new(target_root.path().into());
    let restored = target.import_json(&exported, 0).unwrap();
    assert_eq!(restored.schema_version, 1);
    assert_eq!(
        restored.snippets.last().unwrap().rich_body,
        saved.snippets[0].rich_body
    );

    document["schemaVersion"] = 1.into();
    error_code(target.preview_import(&document.to_string()), "invalidData");
    document["snippets"][0]["richBody"] = serde_json::Value::Null;
    error_code(target.preview_import(&document.to_string()), "invalidData");
    document["schemaVersion"] = 2.into();
    document["snippets"][0]["richBody"] = "invalid".into();
    error_code(target.preview_import(&document.to_string()), "invalidData");
}

#[test]
fn rich_backup_manifest_and_bare_library_v2_round_trip() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    input.snippets[0].body = "Hello".into();
    input.snippets[0].rich_body = Some(r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}}"#.into());
    store.save(input).unwrap();
    let bare = store.export_json().unwrap();
    PreparedImport::decode(bare.as_bytes(), BackupFormat::Json).unwrap();
    let bare_value: serde_json::Value = serde_json::from_str(&bare).unwrap();
    let bare_yaml = serde_yaml_ng::to_string(&bare_value).unwrap();
    PreparedImport::decode(bare_yaml.as_bytes(), BackupFormat::Yaml).unwrap();
    for format in [
        BackupFormat::Json,
        BackupFormat::Yaml,
        BackupFormat::Archive,
    ] {
        let data = store.export_backup(format).unwrap();
        let prepared = PreparedImport::decode(&data, format).unwrap();
        let target_root = tempfile::tempdir().unwrap();
        let target = LibraryStore::new(target_root.path().into());
        let applied = prepared.apply(&target, 0, ImportMode::Replace).unwrap();
        assert_eq!(applied.schema_version, 1);
        assert_eq!(
            applied.snippets[0].rich_body,
            store.load().unwrap().snippets[0].rich_body
        );
    }
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&store.export_backup(BackupFormat::Json).unwrap()).unwrap();
    assert_eq!(manifest["schemaVersion"], 2);
    assert_eq!(manifest["library"]["schemaVersion"], 2);
    manifest["library"]["schemaVersion"] = 1.into();
    assert!(PreparedImport::decode(manifest.to_string().as_bytes(), BackupFormat::Json).is_err());
    manifest["library"]["snippets"][0]["richBody"] = serde_json::Value::Null;
    assert!(PreparedImport::decode(manifest.to_string().as_bytes(), BackupFormat::Json).is_err());
    let invalid_yaml = serde_yaml_ng::to_string(&manifest).unwrap();
    assert!(PreparedImport::decode(invalid_yaml.as_bytes(), BackupFormat::Yaml).is_err());
    manifest["library"]["schemaVersion"] = 2.into();
    manifest["library"]["snippets"][0]["richBody"] = "invalid".into();
    assert!(PreparedImport::decode(manifest.to_string().as_bytes(), BackupFormat::Json).is_err());
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
            rich_body: None,
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
            rich_body: None,
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
                rich_body: None,
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
    connection
        .execute_batch("PRAGMA user_version=999;")
        .unwrap();
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
fn windows_v1_accepts_unicode_triggers_empty_and_large_literal_bodies() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    input.snippets[0].trigger = ":réponse".into();
    input.snippets[0].body.clear();
    let empty = store
        .save(input)
        .expect("empty replacement is a valid deletion snippet");
    assert_eq!(empty.snippets[0].body, "");
    let mut input = SaveLibrary {
        expected_revision: empty.revision,
        folders: empty.folders,
        snippets: empty.snippets,
    };
    input.snippets[0].body = "Café 👋\t\n".repeat(10_000);
    let saved = store
        .save(input.clone())
        .expect("large literal body must persist");
    assert_eq!(saved.snippets[0].body, input.snippets[0].body);
}

#[test]
fn metadata_migrates_v1_and_usage_does_not_stale_an_editor_revision() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let original = store.load().unwrap();
    let conn = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    conn.execute_batch("DROP TABLE IF EXISTS snippet_info; ALTER TABLE snippets DROP COLUMN rich_body; PRAGMA user_version=1;")
        .unwrap();
    drop(conn);
    let metadata = store.metadata().unwrap();
    assert_eq!(metadata.len(), original.snippets.len());
    let id = original.snippets[0].id.clone();
    store.set_favorite(&id, true).unwrap();
    store.record_usage(&id).unwrap();
    assert_eq!(store.load().unwrap(), original);
    let changed = store
        .metadata()
        .unwrap()
        .into_iter()
        .find(|info| info.snippet_id == id)
        .unwrap();
    assert!(changed.favorite);
    assert_eq!(changed.usage_count, 1);
    assert!(changed.created_at > 0);
    assert!(changed.last_used_at.is_some());
    store
        .save(SaveLibrary {
            expected_revision: original.revision,
            folders: original.folders,
            snippets: original.snippets,
        })
        .unwrap();
    assert!(
        store
            .metadata()
            .unwrap()
            .iter()
            .find(|info| info.snippet_id == id)
            .unwrap()
            .favorite
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

#[test]
fn rich_document_survives_save_restart_and_json_import() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = serde_json::to_value(fixture(0)).unwrap();
    let document = r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Café"}]}]}}"#;
    input["snippets"][0]["body"] = "Café".into();
    input["snippets"][0]["richBody"] = document.into();
    let input: SaveLibrary = serde_json::from_value(input).unwrap();
    let saved = store.save(input).unwrap();
    drop(store);
    let store = LibraryStore::new(root.path().into());
    assert_eq!(store.load().unwrap(), saved);
    let exported = store.export_json().unwrap();
    let value: serde_json::Value = serde_json::from_str(&exported).unwrap();
    assert_eq!(value["snippets"][0]["richBody"], document);
    assert_eq!(value["snippets"][0]["body"], "Café");
    let target = tempfile::tempdir().unwrap();
    let target = LibraryStore::new(target.path().into());
    let imported = target.import_json(&exported, 0).unwrap();
    assert_eq!(imported.snippets.last().unwrap(), &saved.snippets[0]);
}

#[test]
fn legacy_databases_migrate_without_changing_literal_text_or_backup_bytes() {
    for version in 1..=3 {
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let saved = store.save(fixture(0)).unwrap();
        let legacy_json = store.export_json().unwrap();
        assert!(!legacy_json.contains("richBody"));
        let backup_dir = root.path().join("backups");
        std::fs::write(backup_dir.join("library-r1.json"), legacy_json.as_bytes()).unwrap();
        let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
        connection
            .execute_batch("ALTER TABLE snippets DROP COLUMN rich_body;")
            .unwrap();
        if version < 3 {
            connection
                .execute_batch("DROP TABLE attachments; DROP TABLE file_settings;")
                .unwrap();
        }
        if version < 2 {
            connection
                .execute_batch("DROP TABLE snippet_info;")
                .unwrap();
        }
        connection
            .pragma_update(None, "user_version", version)
            .unwrap();
        drop(connection);
        assert_eq!(store.load().unwrap(), saved);
        let snapshots: Vec<_> = std::fs::read_dir(&backup_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("library-before-v4-")
            })
            .collect();
        assert_eq!(snapshots.len(), 1);
        let snapshot = rusqlite::Connection::open(&snapshots[0]).unwrap();
        assert_eq!(
            snapshot
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            version
        );
        assert_eq!(store.export_json().unwrap(), legacy_json);
        store
            .save(SaveLibrary {
                expected_revision: saved.revision,
                folders: saved.folders,
                snippets: saved.snippets,
            })
            .unwrap();
        let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
                .unwrap(),
            4
        );
    }
}

#[test]
fn rich_json_is_bounded_and_invalid_json_does_not_change_library() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let before = store.load().unwrap();
    let mut input = fixture(0);
    input.snippets[0].rich_body = Some("{not-json".into());
    error_code(store.save(input.clone()), "invalidData");
    input.snippets[0].rich_body = Some(format!("\"{}\"", "a".repeat(MAX_RICH_BODY_BYTES)));
    error_code(store.save(input), "tooLarge");
    assert_eq!(store.load().unwrap(), before);
}

#[test]
fn rich_documents_must_match_plain_text_and_contain_no_unsafe_links() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let before = store.load().unwrap();
    let mut input = fixture(0);
    input.snippets[0].body = "different".into();
    input.snippets[0].rich_body = Some(r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"visible"}]}]}}"#.into());
    error_code(store.save(input.clone()), "invalidData");
    input.snippets[0].body = "visible".into();
    input.snippets[0].rich_body = Some(r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"visible","marks":[{"type":"link","attrs":{"href":"javascript:alert(1)"}}]}]}]}}"#.into());
    error_code(store.save(input), "invalidData");
    assert_eq!(store.load().unwrap(), before);
}

#[test]
fn imported_rich_content_participates_in_record_conflicts() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let mut input = fixture(0);
    input.snippets[0].body = "Café".into();
    input.snippets[0].rich_body = Some(r#"{"version":1,"doc":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Café"}]}]}}"#.into());
    let saved = store.save(input.clone()).unwrap();
    input.snippets[0].rich_body = None;
    error_code(store.preview_import(&envelope(&input)), "conflict");
    assert_eq!(store.load().unwrap(), saved);
}

#[test]
fn migration_snapshot_preserves_legacy_metadata_and_attachment_references() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    let original = store.load().unwrap();
    let id = &original.snippets[0].id;
    store.set_favorite(id, true).unwrap();
    store.record_usage(id).unwrap();
    let files = store
        .add_attachment_data(id, "example.txt", b"attachment bytes")
        .unwrap();
    store
        .set_file_settings(
            id,
            &FileSettings {
                attachments_first: true,
                strict_order: true,
            },
        )
        .unwrap();
    let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    connection
        .execute_batch("ALTER TABLE snippets DROP COLUMN rich_body; PRAGMA user_version=3;")
        .unwrap();
    drop(connection);
    assert_eq!(store.load().unwrap(), original);
    let backups: Vec<_> = std::fs::read_dir(root.path().join("backups"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("library-before-v4-")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    let snapshot = rusqlite::Connection::open(&backups[0]).unwrap();
    assert_eq!(
        snapshot
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        3
    );
    assert!(
        snapshot
            .query_row("SELECT favorite FROM snippet_info WHERE id=?1", [id], |r| r
                .get::<_, bool>(0))
            .unwrap()
    );
    assert_eq!(
        snapshot
            .query_row(
                "SELECT usage_count FROM snippet_info WHERE id=?1",
                [id],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        snapshot
            .query_row(
                "SELECT digest FROM attachments WHERE snippet_id=?1",
                [id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        files[0].digest
    );
    assert!(
        snapshot
            .query_row(
                "SELECT strict_order FROM file_settings WHERE snippet_id=?1",
                [id],
                |r| r.get::<_, bool>(0)
            )
            .unwrap()
    );
    assert_eq!(
        std::fs::read(root.path().join("attachments").join(&files[0].digest)).unwrap(),
        b"attachment bytes"
    );
}

#[test]
fn failed_migration_snapshot_leaves_legacy_database_untouched() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    store.load().unwrap();
    let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    connection
        .execute_batch("ALTER TABLE snippets DROP COLUMN rich_body; PRAGMA user_version=3;")
        .unwrap();
    std::fs::write(root.path().join("backups"), b"do not overwrite").unwrap();
    error_code(store.load(), "storage");
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        std::fs::read(root.path().join("backups")).unwrap(),
        b"do not overwrite"
    );
}

#[test]
fn migration_snapshot_waits_for_committed_wal_writer_and_never_overwrites_previous_backup() {
    let root = tempfile::tempdir().unwrap();
    let store = LibraryStore::new(root.path().into());
    store.load().unwrap();
    std::fs::create_dir(root.path().join("backups")).unwrap();
    let sentinel = root
        .path()
        .join("backups/library-before-v4-existing.sqlite3");
    std::fs::write(&sentinel, b"previous snapshot").unwrap();
    let connection = rusqlite::Connection::open(root.path().join("library.sqlite3")).unwrap();
    connection.execute_batch("PRAGMA journal_mode=WAL; ALTER TABLE snippets DROP COLUMN rich_body; PRAGMA user_version=3; BEGIN IMMEDIATE; UPDATE snippets SET body='committed by other writer' WHERE trigger=';hello';").unwrap();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (finished_tx, finished_rx) = std::sync::mpsc::channel();
    let path = root.path().to_owned();
    let worker = std::thread::spawn(move || {
        started_tx.send(()).unwrap();
        finished_tx.send(LibraryStore::new(path).load()).unwrap();
    });
    started_rx.recv().unwrap();
    assert!(matches!(
        finished_rx.recv_timeout(std::time::Duration::from_millis(200)),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));
    connection.execute_batch("COMMIT;").unwrap();
    let loaded = finished_rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .unwrap()
        .unwrap();
    worker.join().unwrap();
    assert_eq!(loaded.snippets[0].body, "committed by other writer");
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"previous snapshot");
    let snapshots: Vec<_> = std::fs::read_dir(root.path().join("backups"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path != &sentinel)
        .collect();
    assert_eq!(snapshots.len(), 1);
    let snapshot = rusqlite::Connection::open(&snapshots[0]).unwrap();
    assert_eq!(
        snapshot
            .query_row(
                "SELECT body FROM snippets WHERE trigger=';hello'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "committed by other writer"
    );
    assert_eq!(
        snapshot
            .pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        3
    );
}
