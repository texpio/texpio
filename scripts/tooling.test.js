import assert from 'node:assert/strict'
import { mkdtemp, mkdir, writeFile, readFile, rm, symlink } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import { test } from 'node:test'
import { execFileSync } from 'node:child_process'
import { completeTask, renameDoneTasks } from './complete-task.js'
import { prepareRelease } from './prepare-release.js'

test(
  'Windows launcher refuses running instances and verifies its new window',
  { skip: process.platform !== 'win32' },
  () => {
    const checks = path.join(import.meta.dirname, 'launch-dev-guards.test.ps1')
    const output = execFileSync('pwsh', ['-NoProfile', '-File', checks], {
      encoding: 'utf8',
      timeout: 15000,
    })
    assert.match(output, /Launcher guard checks passed/)
  },
)

test('Windows main window starts hidden until the startup policy shows it', async () => {
  const config = JSON.parse(
    await readFile(new URL('../src-tauri/tauri.windows.conf.json', import.meta.url), 'utf8'),
  )
  assert.equal(config.app.windows.find((window) => window.label === 'main').visible, false)
})

async function fixture(t) {
  const root = await mkdtemp(path.join(os.tmpdir(), 'texpio-tooling-'))
  t.after(() => rm(root, { recursive: true, force: true }))
  await mkdir(path.join(root, 'docs/tasks-todo'), { recursive: true })
  await mkdir(path.join(root, 'docs/tasks-done'), { recursive: true })
  await mkdir(path.join(root, 'src-tauri'))
  return root
}

test('task completion refuses traversal, ambiguous selections and destination collisions', async (t) => {
  const root = await fixture(t)
  const todo = (name) => path.join(root, 'docs/tasks-todo', name)
  await writeFile(todo('task-1-shell.md'), '# Shell')
  await writeFile(todo('task-2-shell-tests.md'), '# Shell tests')
  await assert.rejects(completeTask(root, '../shell'), /identifier/)
  await assert.rejects(completeTask(root, 'shell'), /ambiguous/)
  const destination = path.join(root, 'docs/tasks-done/task-2026-10-24-shell.md')
  await writeFile(destination, 'existing')
  await assert.rejects(completeTask(root, 'task-1-shell.md', '2026-10-24'), /exist/)
  assert.equal(await readFile(destination, 'utf8'), 'existing')
  assert.equal(await readFile(todo('task-1-shell.md'), 'utf8'), '# Shell')
})

test('task completion moves only the uniquely selected regular file', async (t) => {
  const root = await fixture(t)
  await writeFile(path.join(root, 'docs/tasks-todo/task-1-shell.md'), '# Shell')
  await writeFile(path.join(root, 'docs/tasks-todo/task-2-tests.md'), '# Tests')
  const result = await completeTask(root, 'shell', '2026-10-24')
  assert.equal(await readFile(result.destination, 'utf8'), '# Shell')
  assert.equal(
    await readFile(path.join(root, 'docs/tasks-todo/task-2-tests.md'), 'utf8'),
    '# Tests',
  )
  await assert.rejects(readFile(result.source), { code: 'ENOENT' })
})

test('task helper refuses a task directory redirected outside the project', async (t) => {
  const root = await fixture(t)
  const outside = await fixture(t)
  await rm(path.join(root, 'docs/tasks-done'), { recursive: true })
  await symlink(
    path.join(outside, 'docs/tasks-done'),
    path.join(root, 'docs/tasks-done'),
    'junction',
  )
  await writeFile(path.join(root, 'docs/tasks-todo/task-1-shell.md'), '# Shell')
  await assert.rejects(completeTask(root, 'shell'), /contained|symlink/)
})

test('completed-task normalization preflights collisions and leaves already dated files alone', async (t) => {
  const root = await fixture(t)
  const done = (name) => path.join(root, 'docs/tasks-done', name)
  await writeFile(done('task-1-shell.md'), '# Shell')
  await writeFile(done('task-2026-10-24-stable.md'), '# Stable')
  const preview = await renameDoneTasks(root, { dryRun: true })
  assert.equal(preview.length, 1)
  await writeFile(preview[0].destination, 'collision')
  await assert.rejects(renameDoneTasks(root), /exist/)
  assert.equal(await readFile(done('task-1-shell.md'), 'utf8'), '# Shell')
  await rm(preview[0].destination)
  await renameDoneTasks(root)
  assert.equal(await readFile(preview[0].destination, 'utf8'), '# Shell')
  assert.equal(await readFile(done('task-2026-10-24-stable.md'), 'utf8'), '# Stable')
})

async function releaseFixture(t) {
  const root = await fixture(t)
  const files = {
    'package.json': '{"name":"texpio","version":"0.1.0"}\n',
    'src-tauri/tauri.conf.json': '{"version":"0.1.0"}\n',
    'src-tauri/Cargo.toml':
      '[package]\nname = "texpio"\nversion = "0.1.0"\n\n[dependencies]\nserde = "1"\n',
    'src-tauri/Cargo.lock':
      '[[package]]\nname = "texpio"\nversion = "0.1.0"\n\n[[package]]\nname = "serde"\nversion = "1.0.0"\n',
    'bun.lock': '{"lockfileVersion":1,"workspaces":{"":{"name":"texpio"}},"packages":{}}\n',
  }
  for (const [name, body] of Object.entries(files)) await writeFile(path.join(root, name), body)
  return { root, files }
}

test('release dry run changes nothing and never executes a subprocess', async (t) => {
  const { root, files } = await releaseFixture(t)
  const result = await prepareRelease(root, 'v0.2.0', { dryRun: true })
  assert.equal(result.version, '0.2.0')
  for (const [name, body] of Object.entries(files)) {
    assert.equal(await readFile(path.join(root, name), 'utf8'), body)
  }
  await assert.rejects(prepareRelease(root, '0.2.0;git push'), /version/)
  await assert.rejects(prepareRelease(root, '0.2.0-'), /version/)
})

test('release preparation accepts prerelease versions such as an alpha', async (t) => {
  const { root, files } = await releaseFixture(t)
  const result = await prepareRelease(root, 'v0.1.0-alpha.1', { dryRun: true })
  assert.equal(result.version, '0.1.0-alpha.1')
  for (const [name, body] of Object.entries(files)) {
    assert.equal(await readFile(path.join(root, name), 'utf8'), body)
  }
})

test('release preparation updates only project version records including Cargo.lock', async (t) => {
  const { root } = await releaseFixture(t)
  await prepareRelease(root, '0.2.0')
  assert.equal(JSON.parse(await readFile(path.join(root, 'package.json'), 'utf8')).version, '0.2.0')
  assert.equal(
    JSON.parse(await readFile(path.join(root, 'src-tauri/tauri.conf.json'), 'utf8')).version,
    '0.2.0',
  )
  assert.match(
    await readFile(path.join(root, 'src-tauri/Cargo.lock'), 'utf8'),
    /name = "texpio"\nversion = "0.2.0"/,
  )
  assert.match(
    await readFile(path.join(root, 'src-tauri/Cargo.lock'), 'utf8'),
    /name = "serde"\nversion = "1.0.0"/,
  )
  assert.match(await readFile(path.join(root, 'src-tauri/Cargo.toml'), 'utf8'), /serde = "1"/)
})

test('release preparation refuses inconsistent versions before making any changes', async (t) => {
  const { root, files } = await releaseFixture(t)
  await writeFile(path.join(root, 'src-tauri/tauri.conf.json'), '{"version":"0.3.0"}')
  await assert.rejects(prepareRelease(root, '0.2.0'), /inconsistent/)
  assert.equal(await readFile(path.join(root, 'package.json'), 'utf8'), files['package.json'])
})
