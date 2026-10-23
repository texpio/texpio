import { readdir, copyFile, unlink, lstat } from 'node:fs/promises'
import { constants } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { projectPath } from './project-paths.js'

async function moveExclusive(source, destination) {
  await copyFile(source, destination, constants.COPYFILE_EXCL)
  try {
    await unlink(source)
  } catch (error) {
    throw new Error('Completed copy saved but source could not be removed', { cause: error })
  }
}

export async function renameDoneTasks(root, { dryRun = false } = {}) {
  const done = await projectPath(root, 'docs/tasks-done', { directory: true })
  const changes = []
  const destinations = new Set()
  for (const name of await readdir(done)) {
    if (!name.endsWith('.md') || /^task-\d{4}-\d{2}-\d{2}-/.test(name)) continue
    const source = await projectPath(root, path.join('docs/tasks-done', name))
    const date = (await lstat(source)).mtime.toISOString().slice(0, 10)
    const suffix = name.replace(/^task-(?:[0-9x]+-)?/, '')
    const destination = path.join(done, `task-${date}-${suffix}`)
    if (destinations.has(destination.toLowerCase())) throw new Error('Destination collision exists')
    destinations.add(destination.toLowerCase())
    await lstat(destination).then(
      () => {
        throw new Error('Destination already exists')
      },
      (error) => {
        if (error.code !== 'ENOENT') throw error
      },
    )
    changes.push({ source, destination })
  }
  if (!dryRun) for (const change of changes) await moveExclusive(change.source, change.destination)
  return changes
}

export async function completeTask(root, identifier, date = new Date().toISOString().slice(0, 10)) {
  if (!/^[a-z0-9][a-z0-9 ._-]*$/i.test(identifier) || identifier.includes('..')) {
    throw new Error('Invalid task identifier; use a filename or unique name')
  }
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) throw new Error('Invalid completion date')
  const todo = await projectPath(root, 'docs/tasks-todo', { directory: true })
  const done = await projectPath(root, 'docs/tasks-done', { directory: true })
  const files = (await readdir(todo)).filter((name) => name.endsWith('.md'))
  const query = identifier.toLowerCase()
  const exact = files.filter((name) => name.toLowerCase() === query)
  const matches = exact.length ? exact : files.filter((name) => name.toLowerCase().includes(query))
  if (matches.length !== 1)
    throw new Error(matches.length ? 'Task selection is ambiguous' : 'Task not found')
  const source = await projectPath(root, path.join('docs/tasks-todo', matches[0]))
  const name = matches[0].replace(/^task-(?:\d{4}-\d{2}-\d{2}|[0-9x]+)-/, '').replace(/^task-/, '')
  const destination = path.join(done, `task-${date}-${name}`)
  // COPYFILE_EXCL preserves existing completed work even if another process creates it now.
  await moveExclusive(source, destination)
  return { source, destination }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2)
  const root = path.resolve(fileURLToPath(new URL('..', import.meta.url)))
  const operation =
    args[0] === '--rename-existing'
      ? renameDoneTasks(root, { dryRun: args.includes('--dry-run') })
      : completeTask(root, args.join(' '))
  operation
    .then((result) => console.log(JSON.stringify(result)))
    .catch((error) => {
      console.error(error.message)
      process.exitCode = 1
    })
}
