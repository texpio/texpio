use super::*;
#[derive(Clone, Debug, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStatus {
    pub can_recover: bool,
}
impl LibraryStore {
    pub fn recovery_status(&self) -> Result<RecoveryStatus, LibraryError> {
        let _guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        Ok(RecoveryStatus {
            can_recover: verified_corruption(&self.root.join("library.sqlite3"))?,
        })
    }
    pub fn recover_corrupt(&self, confirmed: bool) -> Result<Library, LibraryError> {
        let guard = self.operation.lock().map_err(|_| LibraryError::storage())?;
        if !confirmed || !verified_corruption(&self.root.join("library.sqlite3"))? {
            return Err(LibraryError::invalid_data());
        }
        let mut paths = Vec::new();
        for name in [
            "library.sqlite3",
            "library.sqlite3-wal",
            "library.sqlite3-shm",
        ] {
            let source = self.root.join(name);
            match fs::symlink_metadata(&source) {
                Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
                    paths.push((source, name))
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::NotFound
                        && name != "library.sqlite3" => {}
                _ => return Err(LibraryError::storage()),
            }
        }
        let directory = self
            .root
            .join("recovery")
            .join(format!("library-corrupt-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).map_err(|_| LibraryError::storage())?;
        let mut moved: Vec<(PathBuf, PathBuf)> = Vec::new();
        for (source, name) in paths {
            let destination = directory.join(name);
            if fs::rename(&source, &destination).is_err() {
                for (source, destination) in moved.into_iter().rev() {
                    let _ = fs::rename(destination, source);
                }
                return Err(LibraryError::storage());
            }
            moved.push((source, destination));
        }
        // Every original file is now preserved. A failed initialization never
        // deletes that evidence or overwrites it with a partial new database.
        drop(guard);
        self.load()
    }
}
fn verified_corruption(path: &std::path::Path) -> Result<bool, LibraryError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(_) => return Err(LibraryError::storage()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(LibraryError::storage());
    }
    if metadata.len() == 0 {
        return Ok(false);
    }
    let mut header = Vec::new();
    fs::File::open(path)
        .map_err(|_| LibraryError::storage())?
        .take(16)
        .read_to_end(&mut header)
        .map_err(|_| LibraryError::storage())?;
    if header != b"SQLite format 3\0" {
        return Ok(true);
    }
    let connection = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_| LibraryError::storage())?;
    connection
        .busy_timeout(Duration::from_millis(100))
        .map_err(|_| LibraryError::storage())?;
    match connection.query_row("PRAGMA quick_check(1)", [], |r| r.get::<_, String>(0)) {
        Ok(result) => Ok(result != "ok"),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if matches!(
                error.code,
                rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
            ) =>
        {
            Ok(true)
        }
        Err(_) => Err(LibraryError::storage()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrupt_recovery_requires_confirmation_and_preserves_db_wal_shm() {
        let root = tempfile::tempdir().unwrap();
        for (name, data) in [
            ("library.sqlite3", b"owned corrupt database".as_slice()),
            ("library.sqlite3-wal", b"owned wal"),
            ("library.sqlite3-shm", b"owned shm"),
        ] {
            fs::write(root.path().join(name), data).unwrap();
        }
        let store = LibraryStore::new(root.path().into());
        assert!(store.recovery_status().unwrap().can_recover);
        assert!(store.recover_corrupt(false).is_err());
        assert_eq!(
            fs::read(root.path().join("library.sqlite3")).unwrap(),
            b"owned corrupt database"
        );
        let recovered = store.recover_corrupt(true).unwrap();
        assert!(!recovered.snippets.is_empty());
        let folder = fs::read_dir(root.path().join("recovery"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(
            fs::read(folder.join("library.sqlite3")).unwrap(),
            b"owned corrupt database"
        );
        assert_eq!(
            fs::read(folder.join("library.sqlite3-wal")).unwrap(),
            b"owned wal"
        );
        assert_eq!(
            fs::read(folder.join("library.sqlite3-shm")).unwrap(),
            b"owned shm"
        );
    }
    #[test]
    fn healthy_future_schema_and_non_file_failures_are_never_quarantined() {
        let root = tempfile::tempdir().unwrap();
        let store = LibraryStore::new(root.path().into());
        let original = store.load().unwrap();
        assert!(!store.recovery_status().unwrap().can_recover);
        assert!(store.recover_corrupt(true).is_err());
        assert_eq!(store.load().unwrap(), original);
        let connection = Connection::open(root.path().join("library.sqlite3")).unwrap();
        connection.execute_batch("PRAGMA user_version=999").unwrap();
        drop(connection);
        assert!(!store.recovery_status().unwrap().can_recover);
        assert!(store.recover_corrupt(true).is_err());
        assert!(!root.path().join("recovery").exists());
        let other = tempfile::tempdir().unwrap();
        fs::create_dir(other.path().join("library.sqlite3")).unwrap();
        let inaccessible = LibraryStore::new(other.path().into());
        assert!(inaccessible.recovery_status().is_err());
        assert!(inaccessible.recover_corrupt(true).is_err());
        assert!(!other.path().join("recovery").exists());
    }
}
