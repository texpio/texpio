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
    temporary.persist(path).map_err(|_| ServiceError::Io)?;
    Ok(())
}

pub async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, ServiceError> + Send + 'static,
) -> Result<T, ServiceError> {
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|_| ServiceError::Unavailable)?
}
