// Pure authentication policy. No GNOME, filesystem, process, or device access.
export const MAX_ATTESTATION = 512
export const MAX_PROC_STAT = 4096
export const ATTESTATION_DIRECTORY = '/run/texpio-attest'
const integer = (value) => Number.isSafeInteger(value) && value >= 0 && value <= 0xffffffff
export function trustedDirectory(meta) {
  return (
    meta?.uid === 0 &&
    integer(meta.mode) &&
    meta.symlink === false &&
    (meta.mode & 0o170000) === 0o040000 &&
    (meta.mode & 0o022) === 0
  )
}
export function trustedFile(meta) {
  return (
    meta?.uid === 0 &&
    integer(meta.mode) &&
    meta.symlink === false &&
    meta.links === 1 &&
    (meta.mode & 0o170000) === 0o100000 &&
    (meta.mode & 0o7777) === 0o444
  )
}
const startValid = (value) =>
  typeof value === 'string' &&
  /^[1-9][0-9]{0,19}$/.test(value) &&
  BigInt(value) <= 18446744073709551615n
export function procStartTime(stat, pid) {
  if (
    typeof stat !== 'string' ||
    stat.length > MAX_PROC_STAT ||
    !integer(pid) ||
    pid === 0 ||
    pid > 0x7fffffff
  )
    return null
  const open = stat.indexOf('(')
  const close = stat.lastIndexOf(')')
  if (open < 0 || close <= open || stat.slice(0, open).trim() !== String(pid)) return null
  const start = stat
    .slice(close + 1)
    .trim()
    .split(/\s+/)[19]
  return startValid(start) ? start : null
}
export function validateRecord(raw, pid, uid, stat) {
  if (
    typeof raw !== 'string' ||
    raw.length === 0 ||
    new TextEncoder().encode(raw).length > MAX_ATTESTATION ||
    !integer(uid) ||
    uid === 0
  )
    return false
  try {
    const record = JSON.parse(raw)
    if (
      !record ||
      Array.isArray(record) ||
      Object.keys(record).length !== 4 ||
      !Object.keys(record).every((key) =>
        ['version', 'workerPid', 'sessionUid', 'startTime'].includes(key),
      )
    )
      return false
    return (
      record.version === 1 &&
      record.workerPid === pid &&
      record.sessionUid === uid &&
      startValid(record.startTime) &&
      procStartTime(stat, pid) === record.startTime
    )
  } catch (_) {
    return false
  }
}
