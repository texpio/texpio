//! Device-free validation shared with the root broker's filesystem boundary.
use serde::{Deserialize, Serialize};
pub const MAX_ATTESTATION: usize = 512;
pub const MAX_PROC_STAT: usize = 4096;
pub const DIRECTORY: &str = "/run/texpio-attest";
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Attestation {
    pub version: u32,
    pub worker_pid: u32,
    pub session_uid: u32,
    pub start_time: String,
}
pub fn trusted_directory(uid: u32, mode: u32, symlink: bool) -> bool {
    uid == 0 && !symlink && mode & 0o170000 == 0o040000 && mode & 0o022 == 0
}
pub fn trusted_file(uid: u32, mode: u32, symlink: bool, links: u64) -> bool {
    uid == 0 && !symlink && mode & 0o170000 == 0o100000 && mode & 0o7777 == 0o444 && links == 1
}
fn start_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && !value.starts_with('0')
        && value.bytes().all(|b| b.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}
pub fn proc_start_time(stat: &str, pid: u32) -> Option<&str> {
    if stat.len() > MAX_PROC_STAT || pid == 0 || pid > i32::MAX as u32 {
        return None;
    }
    let open = stat.find('(')?;
    let close = stat.rfind(')')?;
    if close <= open || stat[..open].trim() != pid.to_string() {
        return None;
    }
    let start = stat.get(close + 1..)?.split_whitespace().nth(19)?;
    start_valid(start).then_some(start)
}
pub fn validate_record(bytes: &[u8], pid: u32, uid: u32, stat: &str) -> bool {
    if bytes.is_empty() || bytes.len() > MAX_ATTESTATION || uid == 0 {
        return false;
    }
    let Ok(record) = serde_json::from_slice::<Attestation>(bytes) else {
        return false;
    };
    record.version == crate::VERSION
        && record.worker_pid == pid
        && record.session_uid == uid
        && proc_start_time(stat, pid) == Some(record.start_time.as_str())
}
