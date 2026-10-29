import test from 'node:test'
import assert from 'node:assert/strict'
import {
  trustedDirectory,
  trustedFile,
  procStartTime,
  validateRecord,
  MAX_ATTESTATION,
  MAX_PROC_STAT,
} from './auth-policy.js'
const stat = (pid = 42, start = '123456') =>
  `${pid} (helper name with ) brackets) S ${Array(18).fill('0').join(' ')} ${start} 0 0`
const record = JSON.stringify({ version: 1, workerPid: 42, sessionUid: 1000, startTime: '123456' })
test('only immutable root directories and root read-only regular files are trusted', () => {
  assert.equal(trustedDirectory({ uid: 0, mode: 0o040755, symlink: false }), true)
  assert.equal(trustedFile({ uid: 0, mode: 0o100444, symlink: false, links: 1 }), true)
  for (const metadata of [
    { uid: 1000, mode: 0o040755, symlink: false },
    { uid: 0, mode: 0o040775, symlink: false },
    { uid: 0, mode: 0o040757, symlink: false },
    { uid: 0, mode: 0o040755, symlink: true },
    { uid: 0, mode: 0o100755, symlink: false },
  ])
    assert.equal(trustedDirectory(metadata), false)
  for (const metadata of [
    { uid: 1000, mode: 0o100444, symlink: false, links: 1 },
    { uid: 0, mode: 0o100644, symlink: false, links: 1 },
    { uid: 0, mode: 0o100446, symlink: false, links: 1 },
    { uid: 0, mode: 0o100444, symlink: true, links: 1 },
    { uid: 0, mode: 0o040444, symlink: false, links: 1 },
    { uid: 0, mode: 0o100444, symlink: false, links: 2 },
  ])
    assert.equal(trustedFile(metadata), false)
})
test('kernel starttime parser handles parentheses and refuses malformed or oversized data', () => {
  assert.equal(procStartTime(stat(), 42), '123456')
  for (const value of [stat(41), stat(42, '0003'), '42 (name) S 0', 'x'.repeat(MAX_PROC_STAT + 1)])
    assert.equal(procStartTime(value, 42), null)
})
test('attestation binds daemon-supplied UID and PID to kernel process lifetime', () => {
  assert.equal(validateRecord(record, 42, 1000, stat()), true)
  for (const [raw, pid, uid, kernel] of [
    [record, 43, 1000, stat(43)],
    [record, 42, 1001, stat()],
    [record, 42, 1000, stat(42, '123457')],
    ['{}', 42, 1000, stat()],
    [record.replace('"version":1', '"version":2'), 42, 1000, stat()],
    [record.replace('42', '"42"'), 42, 1000, stat()],
    [record.replace('"123456"', '123456'), 42, 1000, stat()],
    [record.replace('}', ',"extra":1}'), 42, 1000, stat()],
    [' '.repeat(MAX_ATTESTATION + 1), 42, 1000, stat()],
  ])
    assert.equal(validateRecord(raw, pid, uid, kernel), false)
})
