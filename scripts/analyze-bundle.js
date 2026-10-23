import { readdir, stat } from 'node:fs/promises'
import path from 'node:path'

async function report(directory) {
  for (const item of await readdir(directory, { withFileTypes: true })) {
    const file = path.join(directory, item.name)
    if (item.isDirectory()) await report(file)
    else console.log(`${path.relative('dist', file)}: ${(await stat(file)).size} bytes`)
  }
}
await report('dist')
