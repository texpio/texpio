use crate::types::ServiceError;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub fn read_json<T: DeserializeOwned>(path: &Path, limit: usize) -> Result<T, ServiceError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ServiceError::NotFound
        } else {
            ServiceError::Io
        }
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ServiceError::Io);
    }
    if metadata.len() > limit as u64 {
        return Err(ServiceError::DataTooLarge);
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| ServiceError::Io)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ServiceError::Io)?;
    if bytes.len() > limit {
        return Err(ServiceError::DataTooLarge);
    }
    serde_json::from_slice(&bytes).map_err(|_| ServiceError::CorruptData)
}

pub fn write_json<T: Serialize>(path: &Path, data: &T, limit: usize) -> Result<(), ServiceError> {
    let bytes = serde_json::to_vec_pretty(data).map_err(|_| ServiceError::InvalidInput)?;
    if bytes.len() > limit {
        return Err(ServiceError::DataTooLarge);
    }
    let parent = path.parent().ok_or(ServiceError::InvalidInput)?;
    fs::create_dir_all(parent).map_err(|_| ServiceError::Io)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|_| ServiceError::Io)?;
    temporary.write_all(&bytes).map_err(|_| ServiceError::Io)?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|_| ServiceError::Io)?;
    persist_with_retry(temporary, path, std::thread::sleep)
}

fn persist_with_retry(
    mut temporary: tempfile::NamedTempFile,
    path: &Path,
    mut pause: impl FnMut(std::time::Duration),
) -> Result<(), ServiceError> {
    // Windows readers/indexers can briefly deny replacement. Retain the staged
    // file and retry atomically; never delete the destination as a workaround.
    for attempt in 0..=5 {
        match temporary.persist(path) {
            Ok(_) => return Ok(()),
            Err(failure) => {
                if attempt == 5 || !retryable_replace(&failure.error, path) {
                    return Err(ServiceError::Io);
                }
                temporary = failure.file;
                pause(std::time::Duration::from_millis(10));
            }
        }
    }
    Err(ServiceError::Io)
}

fn retryable_replace(error: &std::io::Error, path: &Path) -> bool {
    #[cfg(windows)]
    {
        matches!(error.raw_os_error(), Some(5 | 32 | 33))
            && fs::metadata(path).is_ok_and(|metadata| !metadata.permissions().readonly())
    }
    #[cfg(not(windows))]
    {
        let _ = (error, path);
        false
    }
}

pub async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, ServiceError> + Send + 'static,
) -> Result<T, ServiceError> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| ServiceError::Unavailable)?
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::os::windows::fs::OpenOptionsExt;

    fn replacement(root: &Path) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new_in(root).unwrap();
        file.write_all(br#"{"theme":"dark"}"#).unwrap();
        file.as_file().sync_all().unwrap();
        file
    }

    #[test]
    fn atomic_replace_retries_a_short_windows_delete_sharing_lock() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("preferences.json");
        fs::write(&destination, br#"{"theme":"system"}"#).unwrap();
        let mut held = Some(
            fs::OpenOptions::new()
                .read(true)
                .share_mode(1)
                .open(&destination)
                .unwrap(),
        );
        let mut waits = 0;
        persist_with_retry(replacement(root.path()), &destination, |_| {
            waits += 1;
            assert_eq!(fs::read(&destination).unwrap(), br#"{"theme":"system"}"#);
            drop(held.take());
        })
        .unwrap();
        assert_eq!(waits, 1);
        assert_eq!(fs::read(&destination).unwrap(), br#"{"theme":"dark"}"#);
    }

    #[test]
    fn persistent_windows_lock_is_bounded_and_preserves_previous_bytes() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("preferences.json");
        fs::write(&destination, b"previous").unwrap();
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&destination)
            .unwrap();
        let mut waits = 0;
        assert_eq!(
            persist_with_retry(replacement(root.path()), &destination, |_| waits += 1),
            Err(ServiceError::Io)
        );
        assert_eq!(waits, 5);
        assert_eq!(fs::read(&destination).unwrap(), b"previous");
        drop(held);
    }

    #[test]
    fn readonly_destination_refuses_without_retry_or_data_loss() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("preferences.json");
        fs::write(&destination, b"previous").unwrap();
        let original = fs::metadata(&destination).unwrap().permissions();
        let mut readonly = original.clone();
        readonly.set_readonly(true);
        fs::set_permissions(&destination, readonly).unwrap();
        let mut waits = 0;
        let result = persist_with_retry(replacement(root.path()), &destination, |_| waits += 1);
        fs::set_permissions(&destination, original).unwrap();
        assert_eq!(result, Err(ServiceError::Io));
        assert_eq!(waits, 0);
        assert_eq!(fs::read(&destination).unwrap(), b"previous");
    }
}
