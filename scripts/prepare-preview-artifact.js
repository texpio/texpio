import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { copyFile, lstat, mkdir, readFile, readdir, writeFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { buildTargets } from './platform-builds.js'

async function regularFile(file) {
  const info = await lstat(file)
  if (!info.isFile() || info.size === 0)
    throw new Error(`Expected nonempty regular package: ${file}`)
  return file
}

async function onePackage(directory, matches) {
  const names = await readdir(directory).catch((error) => {
    if (error.code === 'ENOENT') return []
    throw error
  })
  const selected = names.filter(matches)
  if (selected.length !== 1)
    throw new Error(
      `Expected exactly one package in ${directory}; found ${selected.length} (missing or ambiguous)`,
    )
  return regularFile(path.join(directory, selected[0]))
}

function instructions(entry, version, packages) {
  const opening = `# Texpio ${version} — ${entry.id} test preview

This is a CI test preview. Close any running Texpio instance before testing.
The preview shares the normal Texpio library and preferences; back up your
library before testing. Build identity is in build-info.json; SHA256SUMS covers
the included files.

macOS/Linux interactive qualification remains pending. Expansion is disabled
when a field cannot provide the required protection or composition evidence.
Installing the preview or granting permissions does not supply missing evidence.
`
  if (entry.variant === 'windows')
    return `${opening}
## Windows

Run ${packages[0]} on the matching ${entry.id.endsWith('arm64') ? 'ARM64' : 'x64'} Windows system.
This preview installer has no publisher certificate. Windows may ask you to
confirm the publisher before installation. It requires Microsoft WebView2.
`
  if (entry.variant === 'macos')
    return `${opening}
## macOS 14 or later

Extract ${packages[0]} to restore Texpio.app, then move the app to Applications.
Do not run the bare file inside Contents/MacOS. The app uses ad-hoc signing and
is not notarized. After attempting to open your trusted preview, macOS may
require approval in System Settings → Privacy & Security → Open Anyway.
Accessibility and Input Monitoring permissions still require native testing.
`
  const deb = packages.find(
    (name) => name.endsWith('.deb') && !name.startsWith('texpio-wayland-helper_'),
  )
  const image = packages.find((name) => name.endsWith('.AppImage'))
  const linux = `${opening}
## Linux ${entry.variant === 'x11' ? 'X11' : 'Wayland'}

Use a matching ${entry.id.endsWith('arm64') ? 'ARM64' : 'x64'} system and the selected desktop session.
Choose one app package:

- Debian/Ubuntu: install with \`sudo apt install './${deb}'\` to resolve dependencies.
- AppImage: run \`chmod +x './${image}'\`, then launch \`'./${image}'\`.
  The artifact ZIP does not preserve the AppImage executable permission.

Use only one Linux edition at a time; the editions share library/preferences.
`
  return entry.variant === 'wayland'
    ? `${linux}
## Separate Wayland helper

The included helper package targets Ubuntu 24.04 with GNOME 46. Its installation
is optional and requires explicit administrator setup; installing the main app
does not install or start it. Read WAYLAND-HELPER.md and FEASIBILITY.md before
attempting setup on a disposable native qualification host.

The helper remains an incomplete feasibility preview. Ordinary application
expansion is unavailable without the required per-field composition evidence;
broader device permissions do not resolve that limitation.
`
    : linux
}

export async function preparePreviewArtifact({ root = process.cwd(), id, sha }) {
  const entry = buildTargets.find((entry) => entry.id === id)
  if (!entry) throw new Error('Unknown preview variant')
  if (!/^[a-f0-9]{40}$/i.test(sha ?? '')) throw new Error('Expected full commit SHA')
  root = path.resolve(root)
  const { version } = JSON.parse(await readFile(path.join(root, 'package.json'), 'utf8'))
  if (typeof version !== 'string' || !/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version))
    throw new Error('Expected package version')
  const bundle = path.join(root, 'src-tauri/target', entry.target, 'release/bundle')
  const sources = []
  let app
  if (entry.variant === 'windows')
    sources.push(await onePackage(path.join(bundle, 'nsis'), (name) => name.endsWith('.exe')))
  else if (entry.variant === 'macos') {
    app = path.join(bundle, 'macos/Texpio.app')
    if (!(await lstat(app)).isDirectory()) throw new Error('Expected macOS app package directory')
    await regularFile(path.join(app, 'Contents/MacOS/texpio'))
    await regularFile(path.join(app, 'Contents/Info.plist'))
  } else {
    sources.push(await onePackage(path.join(bundle, 'deb'), (name) => name.endsWith('.deb')))
    sources.push(
      await onePackage(path.join(bundle, 'appimage'), (name) => name.endsWith('.AppImage')),
    )
  }
  if (entry.variant === 'wayland') {
    const arch = entry.id.endsWith('arm64') ? 'arm64' : 'amd64'
    sources.push(
      await onePackage(
        path.join(root, 'helper-packages'),
        (name) => name.startsWith('texpio-wayland-helper_') && name.endsWith(`_${arch}.deb`),
      ),
    )
  }

  const directory = path.join(root, 'preview-artifacts', entry.id)
  await mkdir(path.dirname(directory), { recursive: true })
  await mkdir(directory)
  const packages = sources.map((file) => path.basename(file))
  for (const file of sources) await copyFile(file, path.join(directory, path.basename(file)))
  if (app) {
    const archive = `texpio-preview-${entry.id}.app.tar.gz`
    execFileSync(
      'tar',
      ['-czf', path.join(directory, archive), '-C', path.dirname(app), 'Texpio.app'],
      {
        env: { ...process.env, COPYFILE_DISABLE: '1' },
        timeout: 120_000,
      },
    )
    packages.push(archive)
  }
  if (entry.variant === 'wayland') {
    await copyFile(
      path.join(root, 'packaging/linux-wayland/README.md'),
      path.join(directory, 'WAYLAND-HELPER.md'),
    )
    await copyFile(
      path.join(root, 'native/wayland-helper/FEASIBILITY.md'),
      path.join(directory, 'FEASIBILITY.md'),
    )
  }
  await writeFile(path.join(directory, 'README.md'), instructions(entry, version, packages))
  await writeFile(
    path.join(directory, 'build-info.json'),
    `${JSON.stringify(
      {
        id: entry.id,
        target: entry.target,
        variant: entry.variant,
        version,
        sha,
        packages,
      },
      null,
      2,
    )}\n`,
  )
  const files = (await readdir(directory)).sort()
  const sums = []
  for (const name of files) {
    const digest = createHash('sha256')
      .update(await readFile(path.join(directory, name)))
      .digest('hex')
    sums.push(`${digest}  ${name}`)
  }
  await writeFile(path.join(directory, 'SHA256SUMS'), `${sums.join('\n')}\n`)
  return { directory, files: [...files, 'SHA256SUMS'].sort() }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const result = await preparePreviewArtifact({ id: process.argv[2], sha: process.env.GITHUB_SHA })
  process.stdout.write(`Prepared ${result.directory}: ${result.files.join(', ')}\n`)
}
