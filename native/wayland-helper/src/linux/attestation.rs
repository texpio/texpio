//! Root-owned worker attestation. The child PID comes only from fork().
use crate::attestation::{self, Attestation};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
};
pub(super) struct PublishedAttestation {
    path: PathBuf,
}
impl PublishedAttestation {
    pub fn publish(pid: i32, uid: u32) -> io::Result<Self> {
        if pid <= 0 || uid == 0 || unsafe { libc::geteuid() } != 0 {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        // Both ancestors are fixed paths. O_NOFOLLOW verifies they are actual root
        // directories; their modes exclude replacement by a session user.
        for path in ["/run", attestation::DIRECTORY] {
            let directory = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(path)?;
            let metadata = directory.metadata()?;
            if !attestation::trusted_directory(metadata.uid(), metadata.mode(), false) {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        let mut stat = String::new();
        File::open(format!("/proc/{pid}/stat"))?
            .take(attestation::MAX_PROC_STAT as u64 + 1)
            .read_to_string(&mut stat)?;
        let start =
            attestation::proc_start_time(&stat, pid as u32).ok_or(io::ErrorKind::InvalidData)?;
        let record = Attestation {
            version: crate::VERSION,
            worker_pid: pid as u32,
            session_uid: uid,
            start_time: start.into(),
        };
        let bytes = serde_json::to_vec(&record).map_err(io::Error::other)?;
        if bytes.len() > attestation::MAX_ATTESTATION {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let path = PathBuf::from(attestation::DIRECTORY).join(format!("{pid}.json"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o444)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&path)?;
        // Establish cleanup immediately after create_new, including partial writes.
        let published = Self { path };
        file.set_permissions(fs::Permissions::from_mode(0o444))?;
        let metadata = file.metadata()?;
        if !attestation::trusted_file(metadata.uid(), metadata.mode(), false, metadata.nlink()) {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(published)
    }
}
impl Drop for PublishedAttestation {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
