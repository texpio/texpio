import assert from 'node:assert/strict'
import { test } from 'node:test'
import { buildTargets, buildArgs, linuxUpdateManifest } from './platform-builds.js'

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
