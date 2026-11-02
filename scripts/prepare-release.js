import { readFile, writeFile, rename, unlink } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { projectPath } from './project-paths.js'

// This helper has no subprocess or publishing interface. Publication is a separate workflow.
export async function prepareRelease(root, input, { dryRun = false } = {}) {
  const version = input?.replace(/^v/, '')
  // Semver with an optional prerelease suffix, so alpha/beta/rc tags can be
  // prepared without inventing a stable version number for them.
  const identifier = '(?:0|[1-9]\\d*|\\d*[A-Za-z-][0-9A-Za-z-]*)'
  if (
    !new RegExp(
      `^(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)(?:-${identifier}(?:\\.${identifier})*)?$`,
    ).test(version ?? '')
  ) {
    throw new Error(
      'Invalid release version; use a semantic version such as 0.2.0 or 0.2.0-alpha.1',
    )
  }
  const names = [
    'package.json',
    'src-tauri/tauri.conf.json',
    'src-tauri/Cargo.toml',
    'src-tauri/Cargo.lock',
    'bun.lock',
  ]
  const paths = await Promise.all(names.map((name) => projectPath(root, name)))
  const before = await Promise.all(paths.map((name) => readFile(name, 'utf8')))
  const pkg = JSON.parse(before[0])
  const config = JSON.parse(before[1])
  const cargoPackage = /(?<=\[package\][\s\S]*?)^version\s*=\s*"([^"]+)"/m
  const cargoVersion = before[2].match(cargoPackage)?.[1]
  const lockPackage = new RegExp(
    `(\\[\\[package\\]\\]\\r?\\nname = "${pkg.name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}"\\r?\\nversion = ")([^"]+)(")`,
  )
  const lockedVersion = before[3].match(lockPackage)?.[2]
  if (
    !pkg.version ||
    config.version !== pkg.version ||
    cargoVersion !== pkg.version ||
    lockedVersion !== pkg.version
  ) {
    throw new Error('Project versions are missing or inconsistent')
  }
  pkg.version = version
  config.version = version
  const after = [
    `${JSON.stringify(pkg, null, 2)}\n`,
    `${JSON.stringify(config, null, 2)}\n`,
    before[2].replace(cargoPackage, `version = "${version}"`),
    before[3].replace(lockPackage, `$1${version}$3`),
    // Bun does not store this root version today. Preserve its lock byte-for-byte.
    before[4],
  ]
  const changes = names.filter((_, i) => before[i] !== after[i])
  if (!dryRun) {
    const staged = []
    const committed = []
    try {
      for (let i = 0; i < paths.length; i++) {
        if (before[i] === after[i]) continue
        const temporary = `${paths[i]}.release-${process.pid}.tmp`
        await writeFile(temporary, after[i], { flag: 'wx' })
        staged.push({ i, temporary })
      }
      for (const entry of staged) {
        if ((await readFile(paths[entry.i], 'utf8')) !== before[entry.i])
          throw new Error('Project changed during preparation')
        await rename(entry.temporary, paths[entry.i])
        committed.push(entry.i)
      }
    } catch (error) {
      for (const i of committed) await writeFile(paths[i], before[i])
      throw error
    } finally {
      for (const entry of staged) await unlink(entry.temporary).catch(() => {})
    }
  }
  return { version, changes, dryRun }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [version, ...flags] = process.argv.slice(2)
  if (flags.some((flag) => flag !== '--dry-run')) {
    console.error('Usage: bun run release:prepare VERSION [--dry-run]')
    process.exitCode = 1
  } else {
    prepareRelease(path.resolve(fileURLToPath(new URL('..', import.meta.url))), version, {
      dryRun: flags.includes('--dry-run'),
    })
      .then((result) => console.log(JSON.stringify(result)))
      .catch((error) => {
        console.error(error.message)
        process.exitCode = 1
      })
  }
}
