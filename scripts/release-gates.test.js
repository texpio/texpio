import assert from 'node:assert/strict'
import { test } from 'node:test'
import { validateReleaseSetup } from './validate-release.js'

const key = Buffer.from(
  'untrusted comment: minisign public key: A49EF4746FA6E6D7\nRWTX5qZvdPSepDGWg2bz8n6ArQnBMwgQK1geVosorcbLzcunD+KJ+cYe',
).toString('base64')
const config = {
  bundle: { createUpdaterArtifacts: true },
  plugins: { updater: { pubkey: key, endpoints: ['https://updates.texpio.com/latest.json'] } },
}
const secrets = Object.fromEntries(
  [
    'TAURI_SIGNING_PRIVATE_KEY',
    'APPLE_CERTIFICATE',
    'APPLE_CERTIFICATE_PASSWORD',
    'APPLE_SIGNING_IDENTITY',
    'APPLE_ID',
    'APPLE_PASSWORD',
    'APPLE_TEAM_ID',
  ].map((name) => [name, 'fixture']),
)

test('release setup refuses defaults, invalid keys, insecure feeds and missing signing setup', () => {
  assert.throws(() => validateReleaseSetup({}, {}, {}), /updater/)
  assert.throws(
    () =>
      validateReleaseSetup(
        {
          ...config,
          plugins: { updater: { pubkey: 'placeholder', endpoints: ['https://example.com'] } },
        },
        {},
        secrets,
      ),
    /key|feed/,
  )
  assert.throws(() => validateReleaseSetup(config, {}, secrets), /Windows/)
  assert.throws(
    () =>
      validateReleaseSetup(config, { bundle: { windows: { signCommand: 'signing-tool %1' } } }, {}),
    /signing/,
  )
  const http = structuredClone(config)
  http.plugins.updater.endpoints = ['http://updates.texpio.com/latest.json']
  assert.throws(() => validateReleaseSetup(http, {}, secrets), /feed/)
})

test('configured release gate accepts bounded real public configuration without exposing secrets', () => {
  assert.equal(
    validateReleaseSetup(
      config,
      { bundle: { windows: { signCommand: 'signing-tool %1' } } },
      secrets,
    ),
    true,
  )
})
