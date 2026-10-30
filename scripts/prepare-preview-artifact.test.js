import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { chmod, mkdir, mkdtemp, readFile, readdir, rm, stat, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import { test } from 'node:test'
import { buildTargets } from './platform-builds.js'
import { preparePreviewArtifact } from './prepare-preview-artifact.js'

const sha = 'f'.repeat(40)

async function fixture(t, entry) {
  const root = await mkdtemp(path.join(os.tmpdir(), 'texpio-preview-'))
  t.after(() => rm(root, { recursive: true, force: true }))
  const bundle = path.join(root, 'src-tauri/target', entry.target, 'release/bundle')
  const put = async (relative, contents = 'package fixture') => {
    const file = path.join(root, relative)
    await mkdir(path.dirname(file), { recursive: true })
    await writeFile(file, contents)
    return file
  }
  await put('package.json', JSON.stringify({ version: '0.1.0' }))
  const relativeBundle = path.relative(root, bundle)
  if (entry.variant === 'windows') await put(`${relativeBundle}/nsis/Texpio_0.1.0-setup.exe`)
  else if (entry.variant === 'macos') {
    await put(`${relativeBundle}/macos/Texpio.app/Contents/Info.plist`, '<plist/>')
    const executable = await put(`${relativeBundle}/macos/Texpio.app/Contents/MacOS/texpio`)
    await chmod(executable, 0o755)
  } else {
    await put(`${relativeBundle}/deb/texpio_0.1.0.deb`)
    await put(`${relativeBundle}/appimage/texpio_0.1.0.AppImage`)
  }
  await put(`${relativeBundle}/debug-private.pdb`, 'must not upload')
  await put('.env', 'must not upload')
  if (entry.variant === 'wayland') {
    const arch = entry.id.endsWith('arm64') ? 'arm64' : 'amd64'
    await put(`helper-packages/texpio-wayland-helper_0.1.0_${arch}.deb`)
    await put('packaging/linux-wayland/README.md', 'helper installation fixture')
    await put('native/wayland-helper/FEASIBILITY.md', 'readiness fixture')
  }
  return { root, bundle, put }
}

test('all eight preview variants stage only their packages, readiness notes and verifiable identity', async (t) => {
  for (const entry of buildTargets) {
    const { root } = await fixture(t, entry)
    const { directory, files } = await preparePreviewArtifact({ root, id: entry.id, sha })
    assert.equal(directory, path.join(root, 'preview-artifacts', entry.id))
    assert.deepEqual(files.toSorted(), (await readdir(directory)).toSorted())
    assert.ok(files.includes('README.md'))
    assert.ok(files.includes('SHA256SUMS'))
    assert.ok(!files.some((file) => file.includes('.env') || file.includes('.pdb')))
    const info = JSON.parse(await readFile(path.join(directory, 'build-info.json'), 'utf8'))
    assert.equal(info.id, entry.id)
    assert.equal(info.target, entry.target)
    assert.equal(info.variant, entry.variant)
    assert.equal(info.version, '0.1.0')
    assert.equal(info.sha, sha)
    assert.equal(
      info.packages.length,
      entry.variant === 'wayland' ? 3 : entry.variant === 'x11' ? 2 : 1,
    )
    assert.equal(files.includes('WAYLAND-HELPER.md'), entry.variant === 'wayland')
    assert.equal(files.includes('FEASIBILITY.md'), entry.variant === 'wayland')
    const readme = await readFile(path.join(directory, 'README.md'), 'utf8')
    assert.match(readme, /composition/i)
    if (entry.variant === 'wayland') {
      assert.match(readme, /Ubuntu 24\.04/)
      assert.match(readme, /GNOME 46/)
      assert.ok(
        info.packages.some((name) =>
          name.endsWith(`_${entry.id.endsWith('arm64') ? 'arm64' : 'amd64'}.deb`),
        ),
      )
    }
    const checksums = (await readFile(path.join(directory, 'SHA256SUMS'), 'utf8'))
      .trim()
      .split('\n')
    assert.equal(checksums.length, files.length - 1)
    for (const line of checksums) {
      const [, expected, name] = line.match(/^([a-f0-9]{64}) {2}(.+)$/) ?? []
      assert.ok(name)
      assert.equal(
        createHash('sha256')
          .update(await readFile(path.join(directory, name)))
          .digest('hex'),
        expected,
      )
    }
  }
})

test('missing formats, helper architecture and ambiguous packages cannot produce a downloadable preview', async (t) => {
  const entry = buildTargets.find((entry) => entry.id === 'linux-wayland-arm64')
  for (const missing of ['appimage/texpio_0.1.0.AppImage', 'deb/texpio_0.1.0.deb', 'helper']) {
    const { root, bundle } = await fixture(t, entry)
    const file =
      missing === 'helper'
        ? path.join(root, 'helper-packages/texpio-wayland-helper_0.1.0_arm64.deb')
        : path.join(bundle, missing)
    await rm(file)
    await assert.rejects(preparePreviewArtifact({ root, id: entry.id, sha }), /missing|expected/i)
  }
  const windows = buildTargets.find((entry) => entry.id === 'windows-x64')
  const { root, bundle } = await fixture(t, windows)
  await writeFile(path.join(bundle, 'nsis/stale-setup.exe'), 'old package')
  await assert.rejects(preparePreviewArtifact({ root, id: windows.id, sha }), /ambiguous|expected/i)
})

test('invalid identities, nonregular packages and stale staging directories are refused', async (t) => {
  const entry = buildTargets.find((entry) => entry.id === 'windows-x64')
  const { root, bundle } = await fixture(t, entry)
  await assert.rejects(
    preparePreviewArtifact({ root, id: '../windows-x64', sha }),
    /target|variant/i,
  )
  await assert.rejects(
    preparePreviewArtifact({ root, id: entry.id, sha: 'not-a-sha' }),
    /commit|sha/i,
  )
  const setup = path.join(bundle, 'nsis/Texpio_0.1.0-setup.exe')
  await rm(setup)
  await mkdir(setup)
  await assert.rejects(
    preparePreviewArtifact({ root, id: entry.id, sha }),
    /regular|package|expected/i,
  )
  await rm(setup, { recursive: true })
  await writeFile(setup, 'package')
  const directory = path.join(root, 'preview-artifacts', entry.id)
  await mkdir(directory, { recursive: true })
  await writeFile(path.join(directory, 'keep.txt'), 'existing output')
  await assert.rejects(preparePreviewArtifact({ root, id: entry.id, sha }), /exist|staging/i)
  assert.equal(await readFile(path.join(directory, 'keep.txt'), 'utf8'), 'existing output')
})

test('macOS preview tar preserves the app layout and executable permission', async (t) => {
  const entry = buildTargets.find((entry) => entry.id === 'macos-arm64')
  const { root } = await fixture(t, entry)
  const { directory } = await preparePreviewArtifact({ root, id: entry.id, sha })
  const archive = path.join(directory, `texpio-preview-${entry.id}.app.tar.gz`)
  const listing = execFileSync('tar', ['-tzf', archive], { encoding: 'utf8' })
  assert.match(listing, /Texpio\.app\/Contents\/Info\.plist/)
  assert.match(listing, /Texpio\.app\/Contents\/MacOS\/texpio/)
  const extracted = path.join(root, 'extracted')
  await mkdir(extracted)
  execFileSync('tar', ['-xzf', archive, '-C', extracted])
  if (process.platform !== 'win32') {
    const executable = await stat(path.join(extracted, 'Texpio.app/Contents/MacOS/texpio'))
    assert.equal(executable.mode & 0o111, 0o111)
  }
})
