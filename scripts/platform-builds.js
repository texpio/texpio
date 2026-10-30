import { readFile, writeFile, appendFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

export const buildTargets = [
  { id: 'windows-x64', os: 'windows-2025', target: 'x86_64-pc-windows-msvc', variant: 'windows' },
  {
    id: 'windows-arm64',
    os: 'windows-11-arm',
    target: 'aarch64-pc-windows-msvc',
    variant: 'windows',
  },
  { id: 'macos-x64', os: 'macos-15-intel', target: 'x86_64-apple-darwin', variant: 'macos' },
  { id: 'macos-arm64', os: 'macos-14', target: 'aarch64-apple-darwin', variant: 'macos' },
  ...['x11', 'wayland'].flatMap((variant) => [
    { id: `linux-${variant}-x64`, os: 'ubuntu-22.04', target: 'x86_64-unknown-linux-gnu', variant },
    {
      id: `linux-${variant}-arm64`,
      os: 'ubuntu-22.04-arm',
      target: 'aarch64-unknown-linux-gnu',
      variant,
    },
  ]),
]
export function buildArgs(entry, bundle = true) {
  if (
    !buildTargets.some(
      (target) =>
        target.id === entry.id &&
        target.target === entry.target &&
        target.variant === entry.variant,
    )
  )
    throw new Error('Unknown build target')
  const linux = entry.variant === 'x11' || entry.variant === 'wayland'
  const flags = linux
    ? ` --features linux-${entry.variant} --config src-tauri/tauri.linux-${entry.variant}.conf.json`
    : ''
  const bundles = bundle
    ? `--bundles ${linux ? 'appimage,deb' : entry.variant === 'macos' ? 'app,dmg' : 'nsis'}`
    : '--no-bundle'
  return `--target ${entry.target}${flags} ${bundles} -- --locked --no-default-features`
}
export function linuxUpdateManifest(version, variant, assets) {
  if (!/^\d+\.\d+\.\d+$/.test(version) || !['x11', 'wayland'].includes(variant))
    throw new Error('Invalid release identity')
  const platforms = {}
  for (const [arch, names] of [
    ['x86_64', ['amd64', 'x86_64', 'x64']],
    ['aarch64', ['aarch64', 'arm64']],
  ]) {
    const matches = assets.filter((asset) => {
      const name = asset.name.toLowerCase()
      return (
        name.endsWith('.appimage') &&
        name.includes(variant) &&
        names.some((value) => new RegExp(`(?:[_-])${value}(?:[_.-])`).test(name))
      )
    })
    if (matches.length !== 1) throw new Error(`Missing or ambiguous ${variant} ${arch} AppImage`)
    const asset = matches[0]
    const signature = assets.find((item) => item.name === `${asset.name}.sig`)?.signature?.trim()
    if (
      !signature ||
      signature.length > 4096 ||
      !asset.browser_download_url?.startsWith('https://')
    )
      throw new Error('Missing signature or HTTPS asset URL')
    platforms[`linux-${arch}`] = { signature, url: asset.browser_download_url }
  }
  return { version, notes: `Texpio Linux ${variant === 'x11' ? 'X11' : 'Wayland'}`, platforms }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const operation = process.argv[2]
  if (operation === 'matrix') {
    const matrix = {
      include: buildTargets.map((entry) => ({
        ...entry,
        args: buildArgs(entry),
        checkArgs: buildArgs(entry, false),
        previewArgs: buildArgs(entry).replace('--bundles app,dmg', '--bundles app'),
        features:
          entry.variant === 'x11' || entry.variant === 'wayland'
            ? `--features linux-${entry.variant}`
            : '',
      })),
    }
    if (process.env.GITHUB_OUTPUT)
      await appendFile(process.env.GITHUB_OUTPUT, `matrix=${JSON.stringify(matrix)}\n`)
    else process.stdout.write(`${JSON.stringify(matrix)}\n`)
  } else if (operation === 'manifests') {
    const release = JSON.parse(await readFile(process.argv[3], 'utf8'))
    const directory = process.argv[4]
    for (const asset of release.assets) {
      if (asset.name.endsWith('.sig')) {
        if (path.basename(asset.name) !== asset.name || asset.name.includes('\\'))
          throw new Error('Invalid signature asset name')
        asset.signature = await readFile(path.join(directory, asset.name), 'utf8')
      }
    }
    const version = release.tag_name.replace(/^v/, '')
    for (const variant of ['x11', 'wayland'])
      await writeFile(
        `latest-linux-${variant}.json`,
        `${JSON.stringify(linuxUpdateManifest(version, variant, release.assets), null, 2)}\n`,
      )
  } else throw new Error('Use matrix or manifests')
}
