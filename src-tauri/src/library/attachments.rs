use super::*;
use rusqlite::OptionalExtension;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
const MAX_FILE: u64 = 128 * 1024 * 1024;
const MAX_FILES: usize = 100;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Attachment {
    pub id: String,
    pub snippet_id: String,
    pub name: String,
    pub digest: String,
    pub bytes: u64,
    pub position: u32,
    pub present: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileSettings {
    pub attachments_first: bool,
    pub strict_order: bool,
}

impl LibraryStore {
    pub fn list_attachments(&self, id: &str) -> Result<Vec<Attachment>, LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let conn = self.open()?;
        let mut statement=conn.prepare("SELECT id,snippet_id,name,digest,bytes,position FROM attachments WHERE snippet_id=?1 ORDER BY position,id LIMIT 101").map_err(|_|LibraryError::storage())?;
        let files = statement
            .query_map([id], |row| {
                Ok(Attachment {
                    id: row.get(0)?,
                    snippet_id: row.get(1)?,
                    name: row.get(2)?,
                    digest: row.get(3)?,
                    bytes: u64::from(row.get::<_, u32>(4)?),
                    position: row.get(5)?,
                    present: false,
                })
            })
            .map_err(|_| LibraryError::storage())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| LibraryError::storage())?;
        if files.len() > MAX_FILES {
            return Err(LibraryError::too_large());
        }
        files
            .into_iter()
            .map(|mut file| {
                if !valid_leaf(&file.name) || !valid_digest(&file.digest) || file.bytes > MAX_FILE {
                    return Err(LibraryError::invalid_data());
                }
                file.present =
                    fs::symlink_metadata(self.root.join("attachments").join(&file.digest))
                        .is_ok_and(|metadata| {
                            metadata.is_file()
                                && !metadata.file_type().is_symlink()
                                && metadata.len() == file.bytes
                        });
                Ok(file)
            })
            .collect()
    }
    pub fn add_attachments(
        &self,
        id: &str,
        paths: &[PathBuf],
    ) -> Result<Vec<Attachment>, LibraryError> {
        if paths.is_empty() || paths.len() > MAX_FILES {
            return Err(LibraryError::too_large());
        }
        {
            let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
            let mut conn = self.open()?;
            let transaction = conn
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|_| LibraryError::storage())?;
            require_snippet(&transaction, id)?;
            let count = transaction
                .query_row(
                    "SELECT count(*) FROM attachments WHERE snippet_id=?1",
                    [id],
                    |row| row.get::<_, u32>(0),
                )
                .map_err(|_| LibraryError::storage())? as usize;
            if count + paths.len() > MAX_FILES {
                return Err(LibraryError::too_large());
            }
            let next: u32 = transaction
                .query_row(
                    "SELECT coalesce(max(position)+1,0) FROM attachments WHERE snippet_id=?1",
                    [id],
                    |row| row.get(0),
                )
                .map_err(|_| LibraryError::storage())?;
            let mut total = u64::try_from(
                transaction
                    .query_row(
                        "SELECT coalesce(sum(bytes),0) FROM attachments",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|_| LibraryError::storage())?,
            )
            .map_err(|_| LibraryError::storage())?;
            let directory = self.root.join("attachments");
            managed_directory(&directory)?;
            for (offset, path) in paths.iter().enumerate() {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .filter(|name| valid_leaf(name))
                    .ok_or_else(LibraryError::invalid_data)?;
                let mut source = fs::File::open(path).map_err(|_| LibraryError::storage())?;
                let metadata = source.metadata().map_err(|_| LibraryError::storage())?;
                if !metadata.is_file() || metadata.len() > MAX_FILE {
                    return Err(LibraryError::too_large());
                }
                let mut temporary = tempfile::NamedTempFile::new_in(&directory)
                    .map_err(|_| LibraryError::storage())?;
                let (digest, bytes) = copy_hash(&mut source, temporary.as_file_mut())?;
                total = total
                    .checked_add(bytes)
                    .filter(|value| *value <= 512 * 1024 * 1024)
                    .ok_or_else(LibraryError::too_large)?;
                temporary
                    .as_file()
                    .sync_all()
                    .map_err(|_| LibraryError::storage())?;
                let destination = directory.join(&digest);
                if let Err(failure) = temporary.persist_noclobber(&destination) {
                    if !valid_blob(&destination, &digest, bytes) {
                        return Err(LibraryError::storage());
                    }
                    drop(failure);
                }
                transaction.execute("INSERT INTO attachments(id,snippet_id,name,digest,bytes,position) VALUES(?1,?2,?3,?4,?5,?6)",params![uuid::Uuid::new_v4().to_string(),id,name,digest,bytes as i64,next.checked_add(offset as u32).ok_or_else(LibraryError::too_large)?]).map_err(|_|LibraryError::storage())?;
            }
            transaction.commit().map_err(|_| LibraryError::storage())?;
        }
        self.list_attachments(id)
    }
    pub fn remove_attachment(&self, id: &str, attachment: &str) -> Result<(), LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let conn = self.open()?;
        let changed = conn
            .execute(
                "DELETE FROM attachments WHERE id=?1 AND snippet_id=?2",
                params![attachment, id],
            )
            .map_err(|_| LibraryError::storage())?;
        if changed != 1 {
            return Err(LibraryError::invalid_data());
        }
        Ok(())
    }
    pub fn reorder_attachments(&self, id: &str, order: &[String]) -> Result<(), LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let mut conn = self.open()?;
        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| LibraryError::storage())?;
        let old = {
            let mut statement = transaction
                .prepare("SELECT id FROM attachments WHERE snippet_id=?1")
                .map_err(|_| LibraryError::storage())?;
            statement
                .query_map([id], |row| row.get::<_, String>(0))
                .map_err(|_| LibraryError::storage())?
                .collect::<Result<std::collections::HashSet<_>, _>>()
                .map_err(|_| LibraryError::storage())?
        };
        let new: std::collections::HashSet<_> = order.iter().cloned().collect();
        if order.len() != old.len() || old != new {
            return Err(LibraryError::invalid_data());
        }
        for (position, attachment) in order.iter().enumerate() {
            transaction
                .execute(
                    "UPDATE attachments SET position=?1 WHERE id=?2 AND snippet_id=?3",
                    params![position as u32, attachment, id],
                )
                .map_err(|_| LibraryError::storage())?;
        }
        transaction.commit().map_err(|_| LibraryError::storage())?;
        Ok(())
    }
    pub fn file_settings(&self, id: &str) -> Result<FileSettings, LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let conn = self.open()?;
        require_snippet(&conn, id)?;
        conn.query_row(
            "SELECT attachments_first,strict_order FROM file_settings WHERE snippet_id=?1",
            [id],
            |row| {
                Ok(FileSettings {
                    attachments_first: row.get(0)?,
                    strict_order: row.get(1)?,
                })
            },
        )
        .optional()
        .map(|value| value.unwrap_or_default())
        .map_err(|_| LibraryError::storage())
    }
    pub fn set_file_settings(&self, id: &str, settings: &FileSettings) -> Result<(), LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        let conn = self.open()?;
        require_snippet(&conn, id)?;
        conn.execute("INSERT INTO file_settings(snippet_id,attachments_first,strict_order) VALUES(?1,?2,?3) ON CONFLICT(snippet_id) DO UPDATE SET attachments_first=excluded.attachments_first,strict_order=excluded.strict_order",params![id,settings.attachments_first,settings.strict_order]).map_err(|_|LibraryError::storage())?;
        Ok(())
    }
    pub fn delivery_files(&self, id: &str) -> Result<Vec<PathBuf>, LibraryError> {
        let files = self.list_attachments(id)?;
        if files.is_empty() {
            return Ok(Vec::new());
        }
        let directory = self.root.join("deliveries");
        managed_directory(&directory)?;
        files
            .into_iter()
            .map(|file| {
                let blob = self.root.join("attachments").join(&file.digest);
                if !file.present || !valid_blob(&blob, &file.digest, file.bytes) {
                    return Err(LibraryError::storage());
                }
                let directory = directory.join(&file.digest);
                managed_directory(&directory)?;
                let destination = directory.join(&file.name);
                if destination.exists() {
                    if !valid_blob(&destination, &file.digest, file.bytes) {
                        return Err(LibraryError::storage());
                    }
                } else {
                    fs::copy(blob, &destination).map_err(|_| LibraryError::storage())?;
                }
                Ok(destination)
            })
            .collect()
    }
    pub fn add_attachment_data(
        &self,
        id: &str,
        name: &str,
        bytes: &[u8],
    ) -> Result<Vec<Attachment>, LibraryError> {
        if !valid_leaf(name) || bytes.len() > 6 * 1024 * 1024 {
            return Err(LibraryError::invalid_data());
        }
        let temporary = tempfile::tempdir().map_err(|_| LibraryError::storage())?;
        let file = temporary.path().join(name);
        fs::write(&file, bytes).map_err(|_| LibraryError::storage())?;
        self.add_attachments(id, &[file])
    }
}
fn valid_leaf(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    !name.is_empty()
        && name.chars().count() <= 240
        && !name.ends_with(['.', ' '])
        && !name
            .chars()
            .any(|ch| ch.is_control() || "/\\:<>\"|?*".contains(ch))
        && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !['1', '2', '3', '4', '5', '6', '7', '8', '9', '¹', '²', '³']
            .iter()
            .any(|n| stem == format!("COM{n}") || stem == format!("LPT{n}"))
}
fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|ch| ch.is_ascii_digit() || (b'a'..=b'f').contains(&ch))
}
fn require_snippet(conn: &Connection, id: &str) -> Result<(), LibraryError> {
    if conn
        .query_row("SELECT count(*) FROM snippets WHERE id=?1", [id], |row| {
            row.get::<_, u32>(0)
        })
        .map_err(|_| LibraryError::storage())?
        != 1
    {
        return Err(LibraryError::invalid_data());
    }
    Ok(())
}
fn managed_directory(path: &Path) -> Result<(), LibraryError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(LibraryError::storage()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(path).map_err(|_| LibraryError::storage())
        }
        Err(_) => Err(LibraryError::storage()),
    }
}
fn copy_hash(
    source: &mut impl Read,
    destination: &mut impl Write,
) -> Result<(String, u64), LibraryError> {
    let mut hash = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|_| LibraryError::storage())?;
        if read == 0 {
            break;
        }
        count += read as u64;
        if count > MAX_FILE {
            return Err(LibraryError::too_large());
        }
        hash.update(&buffer[..read]);
        destination
            .write_all(&buffer[..read])
            .map_err(|_| LibraryError::storage())?;
    }
    Ok((
        hash.finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        count,
    ))
}
fn valid_blob(path: &Path, digest: &str, bytes: u64) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| {
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() == bytes
    }) && fs::File::open(path)
        .ok()
        .and_then(|mut file| copy_hash(&mut file, &mut std::io::sink()).ok())
        .is_some_and(|(actual, size)| actual == digest && size == bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_storage_deduplicates_content_preserves_names_and_detects_missing_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(directory.path().join("data"));
        let library = store.load().unwrap();
        let id = &library.snippets[0].id;
        let a = directory.path().join("a.txt");
        let b = directory.path().join("résumé.txt");
        std::fs::write(&a, b"owned fixture bytes").unwrap();
        std::fs::write(&b, b"owned fixture bytes").unwrap();
        let added = store.add_attachments(id, &[a, b]).unwrap();
        assert_eq!(added.len(), 2);
        assert_eq!(added[0].digest, added[1].digest);
        assert_eq!(added[1].name, "résumé.txt");
        assert!(added.iter().all(|file| file.present));
        let delivery = store.delivery_files(id).unwrap();
        assert_eq!(delivery.len(), 2);
        assert_eq!(delivery[1].file_name().unwrap(), "résumé.txt");
        assert_eq!(std::fs::read(&delivery[1]).unwrap(), b"owned fixture bytes");
        std::fs::remove_file(store.root.join("attachments").join(&added[0].digest)).unwrap();
        assert!(
            store
                .list_attachments(id)
                .unwrap()
                .iter()
                .all(|file| !file.present)
        );
        assert!(store.delivery_files(id).is_err());
    }
    #[test]
    fn reorder_is_complete_and_only_changes_the_requested_snippet() {
        let directory = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(directory.path().join("data"));
        let id = store.load().unwrap().snippets[0].id.clone();
        let a = directory.path().join("first.txt");
        let b = directory.path().join("second.txt");
        std::fs::write(&a, "first").unwrap();
        std::fs::write(&b, "second").unwrap();
        let files = store.add_attachments(&id, &[a, b]).unwrap();
        let order = vec![files[1].id.clone(), files[0].id.clone()];
        store.reorder_attachments(&id, &order).unwrap();
        assert_eq!(store.list_attachments(&id).unwrap()[0].id, files[1].id);
        assert!(
            store
                .reorder_attachments(&id, &[files[0].id.clone()])
                .is_err()
        );
        store.remove_attachment(&id, &files[1].id).unwrap();
        assert_eq!(store.list_attachments(&id).unwrap().len(), 1);
        store
            .set_file_settings(
                &id,
                &FileSettings {
                    attachments_first: true,
                    strict_order: true,
                },
            )
            .unwrap();
        assert!(store.file_settings(&id).unwrap().strict_order);
    }
    #[test]
    fn attachment_leaf_names_cannot_escape_or_alias_windows_device_names() {
        for name in [
            "../outside",
            "x/y",
            "x\\y",
            "C:target",
            "CON.txt",
            "AUX",
            "trail.",
            "trail ",
            "bad?.txt",
            "LPT¹.txt",
            "",
        ] {
            assert!(!valid_leaf(name), "{name}");
        }
        for name in ["contract.pdf", "résumé.txt", "photo 👋.png"] {
            assert!(valid_leaf(name));
        }
    }
    #[test]
    fn adding_after_removal_appends_after_the_remaining_order() {
        let directory = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(directory.path().join("data"));
        let id = store.load().unwrap().snippets[0].id.clone();
        let paths: Vec<_> = ["first.txt", "second.txt", "third.txt"]
            .iter()
            .map(|name| {
                let path = directory.path().join(name);
                fs::write(&path, name).unwrap();
                path
            })
            .collect();
        let first = store.add_attachments(&id, &paths[..2]).unwrap();
        store.remove_attachment(&id, &first[0].id).unwrap();
        let files = store.add_attachments(&id, &paths[2..]).unwrap();
        let existing = files.iter().find(|file| file.name == "second.txt").unwrap();
        let added = files.iter().find(|file| file.name == "third.txt").unwrap();
        assert!(added.position > existing.position);
    }
}
