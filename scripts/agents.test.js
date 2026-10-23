import assert from 'node:assert/strict'
import { readFile, access } from 'node:fs/promises'
import path from 'node:path'
import { test } from 'node:test'

const root = path.resolve(import.meta.dirname, '..')
const skills = ['check', 'init', 'cleanup', 'change-package-manager']
const roles = ['cleanup-analyzer', 'userguide-reviewer']

test('all four upstream skills and both roles are portable, attributed and locally linked', async () => {
  const index = await readFile(path.join(root, '.agents/README.md'), 'utf8')
  for (const relative of [
    ...skills.map((name) => `.agents/skills/${name}/SKILL.md`),
    ...roles.map((name) => `.agents/agents/${name}.md`),
  ]) {
    const text = await readFile(path.join(root, relative), 'utf8')
    assert.match(text, /1295f691bbdf373896e7c0f1cc969e2040fbf42c/)
    assert.doesNotMatch(text, /allowed-tools:|user-invocable:|\$ARGUMENTS|settings\.local\.json/)
    assert.ok(index.includes(relative.replace('.agents/', '')))
    if (relative.endsWith('SKILL.md')) {
      assert.match(text, /^---\r?\nname: [a-z-]+\r?\ndescription: Use when/m)
    }
    for (const match of text.matchAll(/\]\(([^)]+)\)/g)) {
      if (/^(https?:|#)/.test(match[1])) continue
      await access(path.resolve(path.dirname(path.join(root, relative)), match[1].split('#')[0]))
    }
  }
  assert.match(index, /does not automatically register/)
})

test('release preparation has no subprocess, signing or repository mutation API', async () => {
  const text = await readFile(path.join(root, 'scripts/prepare-release.js'), 'utf8')
  assert.doesNotMatch(
    text,
    /child_process|execSync|spawn\(|git (?:commit|tag|push)|signer generate/,
  )
})
