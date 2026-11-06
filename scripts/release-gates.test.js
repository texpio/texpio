import assert from 'node:assert/strict'
import { test } from 'node:test'
import { unsignedAlphaNotes, validateUnsignedAlpha } from './validate-alpha-release.js'
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

const alpha = {
  version: '0.1.0-alpha.2',
  packageVersion: '0.1.0-alpha.2',
  config: { version: '0.1.0-alpha.2', bundle: { macOS: { minimumSystemVersion: '13.0' } } },
  macosConfig: { bundle: { macOS: { minimumSystemVersion: '13.0' } } },
  windowsConfig: {},
  cargoVersion: '0.1.0-alpha.2',
  lockVersion: '0.1.0-alpha.2',
}

test('unsigned alpha gate accepts a committed prerelease without signing setup', () => {
  assert.equal(validateUnsignedAlpha(alpha), true)
})

test('unsigned alpha gate refuses signing setup, stable versions and drifted versions', () => {
  assert.throws(() => validateUnsignedAlpha({ ...alpha, version: '0.1.0' }), /prerelease/)
  assert.throws(() => validateUnsignedAlpha({ ...alpha, packageVersion: '0.1.0-alpha.1' }), /match/)
  assert.throws(() => validateUnsignedAlpha({ ...alpha, lockVersion: '0.1.0-alpha.1' }), /Cargo/)
  assert.throws(
    () =>
      validateUnsignedAlpha({
        ...alpha,
        config: {
          ...alpha.config,
          bundle: { ...alpha.config.bundle, macOS: { signingIdentity: 'Developer ID' } },
        },
      }),
    /signing identity/,
  )
  assert.throws(
    () =>
      validateUnsignedAlpha({
        ...alpha,
        macosConfig: { bundle: { macOS: { notarize: true } } },
      }),
    /notarization/,
  )
  assert.throws(
    () =>
      validateUnsignedAlpha({
        ...alpha,
        windowsConfig: { bundle: { windows: { signCommand: 'signtool %1' } } },
      }),
    /Windows signing/,
  )
  assert.throws(
    () =>
      validateUnsignedAlpha({
        ...alpha,
        config: {
          ...alpha.config,
          bundle: { ...alpha.config.bundle, createUpdaterArtifacts: true },
        },
      }),
    /updater/,
  )
})

test('unsigned alpha notes explain quarantine approval without claiming notarization', () => {
  const notes = unsignedAlphaNotes('0.1.0-alpha.2')
  assert.match(notes, /0\.1\.0-alpha\.2/)
  assert.match(notes, /xattr -dr com\.apple\.quarantine/)
  assert.match(notes, /not\s+notarized/)
  assert.doesNotMatch(notes, /Developer ID application certificate/)
})
