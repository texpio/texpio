import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { projectPath } from './project-paths.js'

const prerelease = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*$/

/**
 * Unsigned alpha releases must stay publishable without any signing secret.
 * This gate refuses configuration that would silently require a certificate,
 * notarization ticket or Windows signing tool during the build.
 */
export function validateUnsignedAlpha({
  version,
  packageVersion,
  config,
  macosConfig = {},
  windowsConfig = {},
  cargoVersion,
  lockVersion,
}) {
  if (typeof version !== 'string' || !prerelease.test(version))
    throw new Error('Unsigned alpha releases require a prerelease version such as 0.1.0-alpha.2')
  for (const [name, value] of [
    ['package.json', packageVersion],
    ['tauri.conf.json', config?.version],
    ['Cargo.toml', cargoVersion],
    ['Cargo.lock', lockVersion],
  ]) {
    if (value !== undefined && value !== version)
      throw new Error(`${name} must match the committed release version`)
  }
  const mac = { ...config?.bundle?.macOS, ...macosConfig.bundle?.macOS }
  if (typeof mac.signingIdentity === 'string' && mac.signingIdentity.trim())
    throw new Error('Unsigned alpha builds cannot configure a macOS signing identity')
  if (mac.notarize === true)
    throw new Error('Unsigned alpha builds cannot request macOS notarization')
  if (config?.bundle?.createUpdaterArtifacts === true)
    throw new Error('Unsigned alpha builds cannot require updater signatures')
  const signCommand =
    windowsConfig.bundle?.windows?.signCommand ?? config?.bundle?.windows?.signCommand
  if (typeof signCommand === 'string' && signCommand.trim())
    throw new Error('Unsigned alpha builds cannot configure Windows signing')
  return true
}

export function unsignedAlphaNotes(version) {
  return `# Texpio ${version}

Cross-platform alpha built automatically by GitHub Actions. Nothing here is
code-signed or notarized: there is no Apple Developer ID, no certificate and no
signing secret involved, so every platform is published exactly as built.

## Downloads

- **Windows x64** — NSIS \`.exe\` installer and MSI package. SmartScreen may warn
  because the installer has no publisher certificate; choose **More info** →
  **Run anyway**.
- **macOS universal** — one \`.dmg\` for Intel and Apple Silicon. The app is
  ad-hoc signed only (macOS requires that to launch on Apple Silicon) and is not
  notarized.
- **Linux x64** — X11 and Wayland editions, each as \`.deb\` and \`.AppImage\`.
  Install the \`.deb\` with \`sudo apt install\` so dependencies resolve, or make
  the AppImage executable with \`chmod +x\` before launching it.

## macOS first launch

macOS quarantines every app downloaded from the internet, so the first launch is
blocked until you approve it. Either right-click **Texpio** in Finder and choose
**Open**, or clear the quarantine flag:

\`\`\`
xattr -dr com.apple.quarantine /Applications/Texpio.app
\`\`\`

If macOS reports the app as damaged, the download was incomplete or the bundle
was altered; download the DMG again and repeat the command above.

## Known gaps

- Expansion is not physically qualified on macOS or Linux yet: recognition,
  insertion, dead keys, Unicode input, clipboard races and shutdown behaviour
  remain unverified on those hosts.
- No updater feed is configured, so these builds cannot self-update.
- Windows and Linux packages are produced by CI but have not been installed and
  exercised by a maintainer for this alpha.

This is an early alpha. Expect breakage, and please report issues with your
operating system version and keyboard layout.
`
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const [input, ...flags] = process.argv.slice(2)
    const root = await projectPath(process.cwd(), 'package.json')
    const read = (name) => readFile(path.join(path.dirname(root), name), 'utf8')
    const version = input?.replace(/^v/, '')
    const pkg = JSON.parse(await read('package.json'))
    const config = JSON.parse(await read('src-tauri/tauri.conf.json'))
    const macosConfig = JSON.parse(await read('src-tauri/tauri.macos.conf.json'))
    const windowsConfig = JSON.parse(await read('src-tauri/tauri.windows.conf.json'))
    const cargo = await read('src-tauri/Cargo.toml')
    const lock = await read('src-tauri/Cargo.lock')
    const cargoVersion = cargo.match(/(?<=\[package\][\s\S]*?)^version\s*=\s*"([^"]+)"/m)?.[1]
    const lockVersion = lock.match(
      /\[\[package\]\]\r?\nname = "texpio"\r?\nversion = "([^"]+)"/,
    )?.[1]
    validateUnsignedAlpha({
      version,
      packageVersion: pkg.version,
      config,
      macosConfig,
      windowsConfig,
      cargoVersion,
      lockVersion,
    })
    if (flags.includes('--notes')) process.stdout.write(unsignedAlphaNotes(version))
    else console.log('Unsigned alpha gate passed; no signing secrets are required')
  } catch (error) {
    console.error(error.message)
    process.exitCode = 1
  }
}
