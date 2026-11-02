import assert from 'node:assert/strict'
import { test } from 'node:test'
import { buildTargets, buildArgs, linuxUpdateManifest } from './platform-builds.js'
import { execFileSync } from 'node:child_process'

test('CI matrix builds downloadable packages for every native variant', () => {
  const { include } = JSON.parse(
    execFileSync(process.execPath, ['scripts/platform-builds.js', 'matrix'], {
      encoding: 'utf8',
      env: { ...process.env, GITHUB_OUTPUT: '' },
    }),
  )
  assert.equal(include.length, 8)
  for (const entry of include) {
    assert.equal(typeof entry.previewArgs, 'string', `${entry.id} needs preview build arguments`)
    assert.match(entry.previewArgs, new RegExp(`--target ${entry.target}`))
    assert.match(entry.previewArgs, /-- --locked --no-default-features$/)
    assert.doesNotMatch(entry.previewArgs, /--no-bundle/)
    if (entry.variant === 'macos') assert.match(entry.previewArgs, /--bundles app /)
    else if (entry.variant === 'windows') assert.match(entry.previewArgs, /--bundles nsis /)
    else {
      assert.match(entry.previewArgs, /--bundles appimage,deb /)
      assert.match(entry.previewArgs, new RegExp(`--features linux-${entry.variant}`))
      assert.match(entry.previewArgs, new RegExp(`tauri.linux-${entry.variant}.conf.json`))
    }
  }
})

test('release targets separate both Linux editions and all eight native variants', () => {
  assert.equal(buildTargets.length, 8)
  assert.equal(new Set(buildTargets.map((target) => target.id)).size, 8)
  for (const arch of ['x64', 'arm64']) {
    for (const variant of ['x11', 'wayland']) {
      const target = buildTargets.find((entry) => entry.id === `linux-${variant}-${arch}`)
      assert.ok(target)
      const args = buildArgs(target, true)
      assert.match(args, new RegExp(`--features linux-${variant}`))
      assert.match(args, /-- --locked --no-default-features$/)
      assert.ok(target.id.includes(variant))
    }
  }
})
test('release matrix mirrors the RustDesk download layout on 64-bit targets only', () => {
  const { include } = JSON.parse(
    execFileSync(process.execPath, ['scripts/platform-builds.js', 'release-matrix'], {
      encoding: 'utf8',
      env: { ...process.env, GITHUB_OUTPUT: '' },
    }),
  )
  assert.equal(include.length, 7)
  assert.equal(new Set(include.map((entry) => entry.id)).size, 7)
  const windows = include.filter((entry) => entry.variant === 'windows')
  assert.deepEqual(
    windows.map((entry) => entry.id),
    ['windows-x64', 'windows-arm64'],
  )
  for (const entry of windows) assert.match(entry.args, /--bundles nsis,msi /)
  const mac = include.find((entry) => entry.variant === 'macos')
  assert.equal(mac.id, 'macos-universal')
  assert.equal(mac.target, 'universal-apple-darwin')
  assert.equal(mac.rustTargets, 'x86_64-apple-darwin aarch64-apple-darwin')
  assert.match(mac.args, /--bundles app,dmg /)
  const linux = include.filter((entry) => entry.variant === 'x11' || entry.variant === 'wayland')
  assert.equal(linux.length, 4)
  for (const entry of linux) {
    assert.match(entry.args, /--bundles appimage,deb /)
    assert.match(entry.args, new RegExp(`--features linux-${entry.variant}`))
  }
  for (const entry of include) {
    assert.doesNotMatch(entry.target, /i686|i586|armv7/)
    assert.match(entry.args, /-- --locked --no-default-features$/)
  }
})
test('Linux updater feeds cannot mix X11 and Wayland assets', () => {
  const files = [
    'Texpio_X11_0.2.0_amd64.AppImage',
    'Texpio_Wayland_0.2.0_amd64.AppImage',
    'Texpio_X11_0.2.0_aarch64.AppImage',
    'Texpio_Wayland_0.2.0_aarch64.AppImage',
  ]
  const assets = files.flatMap((name) => [
    {
      name,
      browser_download_url: `https://github.com/texpio/texpio/releases/download/v0.2.0/${name}`,
    },
    { name: `${name}.sig`, signature: 'signed-fixture' },
  ])
  for (const variant of ['x11', 'wayland']) {
    const manifest = linuxUpdateManifest('0.2.0', variant, assets)
    assert.equal(Object.keys(manifest.platforms).length, 2)
    for (const platform of Object.values(manifest.platforms))
      assert.match(platform.url.toLowerCase(), new RegExp(variant))
  }
  assert.throws(() => linuxUpdateManifest('0.2.0', 'x11', []), /missing/i)
})
