use std::path::Path;
pub struct LibraryWatch {
    connection: rusqlite::Connection,
    version: i64,
}
impl LibraryWatch {
    pub fn open(root: &Path) -> Option<Self> {
        let connection = rusqlite::Connection::open_with_flags(
            root.join("library.sqlite3"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .ok()?;
        connection
            .busy_timeout(std::time::Duration::from_millis(100))
            .ok()?;
        let version = connection
            .query_row("PRAGMA data_version", [], |row| row.get(0))
            .ok()?;
        Some(Self {
            connection,
            version,
        })
    }
    pub fn changed(&mut self) -> bool {
        let Ok(version) = self
            .connection
            .query_row("PRAGMA data_version", [], |row| row.get::<_, i64>(0))
        else {
            return false;
        };
        let changed = version != self.version;
        self.version = version;
        changed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detects_external_commits_without_treating_polling_as_a_write() {
        let root = tempfile::tempdir().unwrap();
        let store = crate::library::LibraryStore::new(root.path().into());
        let library = store.load().unwrap();
        let mut watch = LibraryWatch::open(root.path()).unwrap();
        assert!(!watch.changed());
        let other = crate::library::LibraryStore::new(root.path().into());
        other.set_favorite(&library.snippets[0].id, true).unwrap();
        assert!(watch.changed());
        assert!(!watch.changed());
        other.record_usage(&library.snippets[0].id).unwrap();
        assert!(watch.changed());
        assert_eq!(store.load().unwrap().revision, library.revision);
    }
}
