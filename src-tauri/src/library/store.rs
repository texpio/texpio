use super::{
    validation::{encode_library, parse_import, validate_records},
    *,
};
#[path = "attachments.rs"]
mod attachments;
pub use attachments::{Attachment, FileSettings};
use rusqlite::{Connection, Transaction, TransactionBehavior, params};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};

/// Each instance serializes its operations. SQLite immediate transactions also
/// serialize writers from other instances/processes before checking revisions.
pub struct LibraryStore {
    root: PathBuf,
    operation: Mutex<()>,
}

impl LibraryStore {
    pub(crate) fn fence_revision(&self, minimum: u32) -> Result<(), LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let mut connection = self.open()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| LibraryError::storage())?;
        let previous = read_library(&transaction)?;
        let revision = previous
            .revision
            .max(minimum)
            .checked_add(1)
            .ok_or_else(LibraryError::conflict)?;
        transaction
            .execute(
                "UPDATE metadata SET revision=?1 WHERE singleton=1",
                [revision],
            )
            .map_err(|_| LibraryError::storage())?;
        transaction.commit().map_err(|_| LibraryError::storage())
    }
    pub(crate) fn root(&self) -> &std::path::Path {
        &self.root
    }
    pub fn metadata(&self) -> Result<Vec<SnippetInfo>, LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let connection = self.open()?;
        let mut statement=connection.prepare("SELECT i.id,i.favorite,i.usage_count,i.created_at,i.updated_at,i.last_used_at FROM snippet_info i JOIN snippets s ON s.id=i.id ORDER BY i.id LIMIT 10001").map_err(|_|LibraryError::storage())?;
        statement
            .query_map([], |row| {
                Ok(SnippetInfo {
                    snippet_id: row.get(0)?,
                    favorite: row.get(1)?,
                    usage_count: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    last_used_at: row.get(5)?,
                })
            })
            .map_err(|_| LibraryError::storage())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| LibraryError::storage())
    }
    pub fn set_favorite(&self, id: &str, favorite: bool) -> Result<(), LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let connection = self.open()?;
        let changed=connection.execute("UPDATE snippet_info SET favorite=?1 WHERE id=?2 AND EXISTS(SELECT 1 FROM snippets WHERE id=?2)",params![favorite,id]).map_err(|_|LibraryError::storage())?;
        if changed != 1 {
            return Err(LibraryError::invalid_data());
        }
        Ok(())
    }
    pub fn record_usage(&self, id: &str) -> Result<(), LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let connection = self.open()?;
        connection.execute("UPDATE snippet_info SET usage_count=min(usage_count+1,4294967295),last_used_at=?1 WHERE id=?2 AND EXISTS(SELECT 1 FROM snippets WHERE id=?2)",params![now_ms(),id]).map_err(|_|LibraryError::storage())?;
        Ok(())
    }
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            operation: Mutex::new(()),
        }
    }

    pub fn load(&self) -> Result<Library, LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let mut connection = self.open()?;
        let transaction = connection
            .transaction()
            .map_err(|_| LibraryError::storage())?;
        let library = read_library(&transaction)?;
        transaction.commit().map_err(|_| LibraryError::storage())?;
        Ok(library)
    }

    pub fn save(&self, input: SaveLibrary) -> Result<Library, LibraryError> {
        validate_records(&input.folders, &input.snippets, true)?;
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let mut connection = self.open()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| LibraryError::storage())?;
        let previous = read_library(&transaction)?;
        if input.expected_revision != previous.revision {
            return Err(LibraryError::conflict());
        }
        let library = Library {
            schema_version: SCHEMA_VERSION,
            revision: previous
                .revision
                .checked_add(1)
                .ok_or_else(LibraryError::conflict)?,
            folders: input.folders,
            snippets: input.snippets,
        };
        // Validate total encoded size before creating a backup or changing SQL.
        encode_library(&library)?;
        self.backup(&previous)?;
        replace_library(&transaction, &library, Some(&previous))?;
        transaction.commit().map_err(|_| LibraryError::storage())?;
        Ok(library)
    }

    pub fn export_json(&self) -> Result<String, LibraryError> {
        encode_library(&self.load()?)
    }

    pub fn preview_import(&self, json: &str) -> Result<ImportPreview, LibraryError> {
        let incoming = parse_import(json)?;
        let current = self.load()?;
        let (merged, preview) = merge_import(&current, incoming)?;
        encode_library(&merged)?;
        Ok(preview)
    }

    pub fn import_json(&self, json: &str, expected_revision: u32) -> Result<Library, LibraryError> {
        let incoming = parse_import(json)?;
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let mut connection = self.open()?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| LibraryError::storage())?;
        let previous = read_library(&transaction)?;
        if expected_revision != previous.revision {
            return Err(LibraryError::conflict());
        }
        let (mut library, preview) = merge_import(&previous, incoming)?;
        if preview.folders == 0 && preview.snippets == 0 {
            transaction.commit().map_err(|_| LibraryError::storage())?;
            return Ok(previous);
        }
        library.revision = previous
            .revision
            .checked_add(1)
            .ok_or_else(LibraryError::conflict)?;
        encode_library(&library)?;
        self.backup(&previous)?;
        replace_library(&transaction, &library, Some(&previous))?;
        transaction.commit().map_err(|_| LibraryError::storage())?;
        Ok(library)
    }

    fn open(&self) -> Result<Connection, LibraryError> {
        fs::create_dir_all(&self.root).map_err(|_| LibraryError::storage())?;
        let mut connection = Connection::open(self.root.join("library.sqlite3"))
            .map_err(|_| LibraryError::storage())?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(|_| LibraryError::storage())?;
        connection
            .execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
            .map_err(|_| LibraryError::storage())?;
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| LibraryError::storage())?;
        let version: u32 = transaction
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|_| LibraryError::storage())?;
        if version > 3 {
            return Err(LibraryError::unsupported_version());
        }
        if version == 0 {
            let tables: u32 = transaction.query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [], |row| row.get(0),
            ).map_err(|_| LibraryError::storage())?;
            if tables != 0 {
                return Err(LibraryError::storage());
            }
            transaction.execute_batch(
                "CREATE TABLE metadata (singleton INTEGER PRIMARY KEY CHECK(singleton=1), revision INTEGER NOT NULL CHECK(revision>=0));
                INSERT INTO metadata(singleton, revision) VALUES(1, 0);
                CREATE TABLE folders (id TEXT PRIMARY KEY NOT NULL, name TEXT NOT NULL, position INTEGER NOT NULL);
                CREATE TABLE snippets (id TEXT PRIMARY KEY NOT NULL, folder_id TEXT REFERENCES folders(id), title TEXT NOT NULL,
                    trigger TEXT NOT NULL, body TEXT NOT NULL, enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
                    deleted INTEGER NOT NULL CHECK(deleted IN (0,1)), position INTEGER NOT NULL);
                CREATE UNIQUE INDEX active_triggers ON snippets(trigger) WHERE enabled=1 AND deleted=0;
                CREATE TABLE snippet_info(id TEXT PRIMARY KEY NOT NULL,favorite INTEGER NOT NULL DEFAULT 0 CHECK(favorite IN(0,1)),usage_count INTEGER NOT NULL DEFAULT 0,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL,last_used_at INTEGER);
                PRAGMA user_version=1;"
            ).map_err(|_| LibraryError::storage())?;
            replace_library(&transaction, &initial_library(), None)?;
        }
        if version < 2 {
            transaction.execute_batch("CREATE TABLE IF NOT EXISTS snippet_info(id TEXT PRIMARY KEY NOT NULL,favorite INTEGER NOT NULL DEFAULT 0 CHECK(favorite IN(0,1)),usage_count INTEGER NOT NULL DEFAULT 0,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL,last_used_at INTEGER); PRAGMA user_version=2;").map_err(|_|LibraryError::storage())?;
        }
        if version < 3 {
            transaction.execute_batch("CREATE TABLE IF NOT EXISTS attachments(id TEXT PRIMARY KEY NOT NULL,snippet_id TEXT NOT NULL,name TEXT NOT NULL,digest TEXT NOT NULL,bytes INTEGER NOT NULL,position INTEGER NOT NULL); CREATE INDEX IF NOT EXISTS attachment_snippet ON attachments(snippet_id,position); CREATE TABLE IF NOT EXISTS file_settings(snippet_id TEXT PRIMARY KEY NOT NULL,attachments_first INTEGER NOT NULL DEFAULT 0,strict_order INTEGER NOT NULL DEFAULT 0); PRAGMA user_version=3;").map_err(|_|LibraryError::storage())?;
        }
        transaction.execute("INSERT OR IGNORE INTO snippet_info(id,created_at,updated_at) SELECT id,?1,?1 FROM snippets",[now_ms()]).map_err(|_|LibraryError::storage())?;
        transaction.commit().map_err(|_| LibraryError::storage())?;
        Ok(connection)
    }

    fn backup(&self, library: &Library) -> Result<(), LibraryError> {
        let json = encode_library(library)?;
        let directory = self.root.join("backups");
        fs::create_dir_all(&directory).map_err(|_| LibraryError::storage())?;
        let destination = directory.join(format!("library-r{}.json", library.revision));
        let mut temporary =
            tempfile::NamedTempFile::new_in(&directory).map_err(|_| LibraryError::storage())?;
        temporary
            .write_all(json.as_bytes())
            .map_err(|_| LibraryError::storage())?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|_| LibraryError::storage())?;
        match temporary.persist_noclobber(&destination) {
            Ok(_) => Ok(()),
            Err(_) => {
                // A previous failed transaction can leave this revision's backup.
                // Reuse it only when it is exactly the same known-good snapshot.
                let metadata =
                    fs::symlink_metadata(&destination).map_err(|_| LibraryError::storage())?;
                if !metadata.is_file()
                    || metadata.file_type().is_symlink()
                    || metadata.len() > MAX_JSON_BYTES as u64
                {
                    return Err(LibraryError::storage());
                }
                let mut bytes = Vec::new();
                fs::File::open(&destination)
                    .map_err(|_| LibraryError::storage())?
                    .take(MAX_JSON_BYTES as u64 + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|_| LibraryError::storage())?;
                if bytes == json.as_bytes() {
                    Ok(())
                } else {
                    Err(LibraryError::storage())
                }
            }
        }
    }
}

fn read_library(connection: &Connection) -> Result<Library, LibraryError> {
    let revision = connection
        .query_row(
            "SELECT revision FROM metadata WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .map_err(|_| LibraryError::storage())?;
    let mut folder_statement = connection
        .prepare("SELECT id, name FROM folders ORDER BY position, id LIMIT 1001")
        .map_err(|_| LibraryError::storage())?;
    let folders = folder_statement
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|_| LibraryError::storage())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| LibraryError::storage())?;
    let mut snippet_statement = connection.prepare(
        "SELECT id, folder_id, title, trigger, body, enabled, deleted FROM snippets ORDER BY position, id LIMIT 10001"
    ).map_err(|_| LibraryError::storage())?;
    let snippets = snippet_statement
        .query_map([], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                title: row.get(2)?,
                trigger: row.get(3)?,
                body: row.get(4)?,
                enabled: row.get(5)?,
                deleted: row.get(6)?,
            })
        })
        .map_err(|_| LibraryError::storage())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| LibraryError::storage())?;
    validate_records(&folders, &snippets, true).map_err(|_| LibraryError::storage())?;
    let library = Library {
        schema_version: SCHEMA_VERSION,
        revision,
        folders,
        snippets,
    };
    encode_library(&library).map_err(|_| LibraryError::storage())?;
    Ok(library)
}

fn replace_library(
    transaction: &Transaction<'_>,
    library: &Library,
    previous: Option<&Library>,
) -> Result<(), LibraryError> {
    transaction
        .execute_batch("DELETE FROM snippets; DELETE FROM folders;")
        .map_err(|_| LibraryError::storage())?;
    {
        let mut statement = transaction
            .prepare("INSERT INTO folders(id,name,position) VALUES(?1,?2,?3)")
            .map_err(|_| LibraryError::storage())?;
        for (position, folder) in library.folders.iter().enumerate() {
            statement
                .execute(params![folder.id, folder.name, position as i64])
                .map_err(|_| LibraryError::storage())?;
        }
    }
    {
        let mut statement = transaction.prepare(
            "INSERT INTO snippets(id,folder_id,title,trigger,body,enabled,deleted,position) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)"
        ).map_err(|_| LibraryError::storage())?;
        for (position, snippet) in library.snippets.iter().enumerate() {
            statement
                .execute(params![
                    snippet.id,
                    snippet.folder_id,
                    snippet.title,
                    snippet.trigger,
                    snippet.body,
                    snippet.enabled,
                    snippet.deleted,
                    position as i64
                ])
                .map_err(|_| LibraryError::storage())?;
        }
    }
    transaction
        .execute(
            "UPDATE metadata SET revision=?1 WHERE singleton=1",
            [library.revision],
        )
        .map_err(|_| LibraryError::storage())?;
    let old: HashMap<_, _> = previous
        .into_iter()
        .flat_map(|library| library.snippets.iter())
        .map(|snippet| (&snippet.id, snippet))
        .collect();
    for snippet in &library.snippets {
        transaction
            .execute(
                "INSERT OR IGNORE INTO snippet_info(id,created_at,updated_at) VALUES(?1,?2,?2)",
                params![snippet.id, now_ms()],
            )
            .map_err(|_| LibraryError::storage())?;
        if old
            .get(&snippet.id)
            .is_none_or(|previous| *previous != snippet)
        {
            transaction
                .execute(
                    "UPDATE snippet_info SET updated_at=?1 WHERE id=?2",
                    params![now_ms(), snippet.id],
                )
                .map_err(|_| LibraryError::storage())?;
        }
    }
    transaction
        .execute(
            "DELETE FROM snippet_info WHERE id NOT IN(SELECT id FROM snippets)",
            [],
        )
        .map_err(|_| LibraryError::storage())?;
    if previous.is_some() {
        transaction.execute_batch("DELETE FROM attachments WHERE snippet_id NOT IN(SELECT id FROM snippets); DELETE FROM file_settings WHERE snippet_id NOT IN(SELECT id FROM snippets);").map_err(|_|LibraryError::storage())?;
    }
    Ok(())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(i64::MAX as u128) as i64
}

fn merge_import(
    current: &Library,
    incoming: Library,
) -> Result<(Library, ImportPreview), LibraryError> {
    let mut merged = current.clone();
    let mut preview = ImportPreview {
        folders: 0,
        snippets: 0,
        skipped: 0,
    };
    let folders: HashMap<_, _> = current
        .folders
        .iter()
        .map(|folder| (folder.id.as_str(), folder))
        .collect();
    let snippets: HashMap<_, _> = current
        .snippets
        .iter()
        .map(|snippet| (snippet.id.as_str(), snippet))
        .collect();
    for folder in incoming.folders {
        if snippets.contains_key(folder.id.as_str()) {
            return Err(LibraryError::conflict());
        }
        if let Some(existing) = folders.get(folder.id.as_str()) {
            if **existing != folder {
                return Err(LibraryError::conflict());
            }
            preview.skipped += 1;
        } else {
            merged.folders.push(folder);
            preview.folders += 1;
        }
    }
    for snippet in incoming.snippets {
        if folders.contains_key(snippet.id.as_str()) {
            return Err(LibraryError::conflict());
        }
        if let Some(existing) = snippets.get(snippet.id.as_str()) {
            if **existing != snippet {
                return Err(LibraryError::conflict());
            }
            preview.skipped += 1;
        } else {
            merged.snippets.push(snippet);
            preview.snippets += 1;
        }
    }
    validate_records(&merged.folders, &merged.snippets, true)?;
    Ok((merged, preview))
}

fn initial_library() -> Library {
    let folder_id = "dc35d1f3-c7bf-40be-ad05-f3bfb5991385";
    Library {
        schema_version: SCHEMA_VERSION,
        revision: 0,
        folders: vec![Folder {
            id: folder_id.into(),
            name: "Getting started".into(),
        }],
        snippets: vec![
            Snippet {
                id: "63a80ed7-a35d-44b2-b90a-73a4d3379d89".into(),
                folder_id: Some(folder_id.into()),
                title: "Greeting".into(),
                trigger: ";hello".into(),
                body: "Hello! Thanks for getting in touch.".into(),
                enabled: true,
                deleted: false,
            },
            Snippet {
                id: "4f1d1731-657f-4fde-950b-2540b13cbf41".into(),
                folder_id: Some(folder_id.into()),
                title: "Example address".into(),
                trigger: ";addr".into(),
                body: "123 Example Street\nSample City".into(),
                enabled: true,
                deleted: false,
            },
        ],
    }
}
