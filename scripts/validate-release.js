import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { prepareRelease } from './prepare-release.js'

export function validateReleaseSetup(config, windowsConfig, environment) {
  const updater = config.plugins?.updater
  if (config.bundle?.createUpdaterArtifacts !== true || !updater)
    throw new Error('Real updater setup required')
  const key =
    typeof updater.pubkey === 'string'
      ? Buffer.from(updater.pubkey, 'base64').toString('utf8').trim()
      : ''
  const packet = Buffer.from(key.split(/\r?\n/).at(-1) ?? '', 'base64')
  if (
    !key.startsWith('untrusted comment:') ||
    packet.length !== 42 ||
    packet.subarray(0, 2).toString() !== 'Ed'
  ) {
    throw new Error('Real minisign public key required')
  }
  if (
    !Array.isArray(updater.endpoints) ||
    !updater.endpoints.length ||
    updater.endpoints.length > 8
  ) {
    throw new Error('Real HTTPS update feed required')
  }
  for (const endpoint of updater.endpoints) {
    let url
    try {
      url = new URL(endpoint)
    } catch {
      throw new Error('Invalid update feed')
    }
    if (
      url.protocol !== 'https:' ||
      url.username ||
      url.password ||
      url.hash ||
      /localhost|example\.|\.invalid$|\.example$|YOUR_/i.test(url.hostname)
    ) {
      throw new Error('Real HTTPS update feed required')
    }
  }
  const signCommand =
    windowsConfig.bundle?.windows?.signCommand ?? config.bundle?.windows?.signCommand
  if (typeof signCommand !== 'string' || !signCommand.trim())
    throw new Error('Windows signing command setup required')
  for (const name of [
    'TAURI_SIGNING_PRIVATE_KEY',
    'APPLE_CERTIFICATE',
    'APPLE_CERTIFICATE_PASSWORD',
    'APPLE_SIGNING_IDENTITY',
    'APPLE_ID',
    'APPLE_PASSWORD',
    'APPLE_TEAM_ID',
  ]) {
    if (!environment[name]?.trim())
      throw new Error('Platform signing and notarization secrets required')
  }
  return true
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    await prepareRelease(process.cwd(), process.env.RELEASE_VERSION, { dryRun: true })
    const pkg = JSON.parse(await readFile('package.json', 'utf8'))
    if (pkg.version !== process.env.RELEASE_VERSION)
      throw new Error('Release version must already be committed')
    const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'))
    const windowsConfig = JSON.parse(await readFile('src-tauri/tauri.windows.conf.json', 'utf8'))
    validateReleaseSetup(config, windowsConfig, process.env)
    console.log('Release configuration gate passed; packaged acceptance still required')
  } catch (error) {
    console.error(error.message)
    process.exitCode = 1
  }
}
