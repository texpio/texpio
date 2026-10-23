import assert from 'node:assert/strict'
import { once } from 'node:events'
import { test } from 'node:test'
import { startProbeFixture } from './serve-probe-fixture.mjs'

test('fixture server binds only loopback and serves only known GET assets', async () => {
  const server = await startProbeFixture(0)
  try {
    const address = server.address()
    assert.equal(address.address, '127.0.0.1')
    const base = `http://127.0.0.1:${address.port}`
    const page = await fetch(base)
    assert.equal(page.status, 200)
    assert.match(await page.text(), /Texpio P0 textarea/)
    assert.match(page.headers.get('content-security-policy'), /connect-src 'none'/)
    const script = await fetch(`${base}/fixture.js`)
    assert.equal(script.status, 200)
    assert.match(script.headers.get('content-type'), /javascript/)
    for (const path of [
      '/package.json',
      '/%2e%2e%2fpackage.json',
      '/unknown',
      '/fixture.js/extra',
    ]) {
      const response = await fetch(base + path)
      assert.equal(response.status, 404, path)
    }
    const post = await fetch(base, { method: 'POST', body: 'synthetic only' })
    assert.equal(post.status, 405)
    assert.equal(post.headers.get('allow'), 'GET')
  } finally {
    server.close()
    await once(server, 'close')
  }
})
