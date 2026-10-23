import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'

const assets = new Map([
  ['/', ['index.html', 'text/html; charset=utf-8']],
  ['/index.html', ['index.html', 'text/html; charset=utf-8']],
  ['/fixture.js', ['fixture.js', 'text/javascript; charset=utf-8']],
])

export async function startProbeFixture(port = 4175) {
  const server = createServer(async (request, response) => {
    response.setHeader('Cache-Control', 'no-store')
    response.setHeader('X-Content-Type-Options', 'nosniff')
    response.setHeader(
      'Content-Security-Policy',
      "default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; connect-src 'none'; form-action 'none'; frame-ancestors 'none'",
    )
    if (request.method !== 'GET') {
      response.writeHead(405, { Allow: 'GET' }).end('GET only')
      return
    }
    const asset = assets.get((request.url ?? '').split('?')[0])
    if (!asset) {
      response.writeHead(404).end('Not found')
      return
    }
    try {
      const body = await readFile(
        new URL(`../tests/fixtures/windows-probe/${asset[0]}`, import.meta.url),
      )
      response.writeHead(200, { 'Content-Type': asset[1] }).end(body)
    } catch {
      response.writeHead(500).end('Fixture unavailable')
    }
  })
  await new Promise((resolve, reject) => {
    server.once('error', reject)
    server.listen(port, '127.0.0.1', resolve)
  })
  return server
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  const server = await startProbeFixture()
  console.log(`Texpio P0 fixture: http://127.0.0.1:${server.address().port}`)
  for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => server.close())
}
