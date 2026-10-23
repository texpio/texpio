import { lstat, realpath } from 'node:fs/promises'
import path from 'node:path'

// Refuse redirected parents as well as final symlinks before moving or rewriting files.
export async function projectPath(root, relative, { directory = false } = {}) {
  const base = await realpath(root)
  const target = path.resolve(base, relative)
  const suffix = path.relative(base, target)
  if (!suffix || suffix.startsWith('..') || path.isAbsolute(suffix)) {
    throw new Error('Path must be contained in the project')
  }
  let current = base
  for (const segment of suffix.split(path.sep)) {
    current = path.join(current, segment)
    const info = await lstat(current)
    if (info.isSymbolicLink()) throw new Error('Project path cannot contain a symlink')
    const resolved = await realpath(current)
    if (path.relative(base, resolved).startsWith('..')) throw new Error('Path must be contained')
  }
  const info = await lstat(target)
  if (directory ? !info.isDirectory() : !info.isFile())
    throw new Error('Unexpected project path type')
  return target
}
