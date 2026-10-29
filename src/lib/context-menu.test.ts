import { beforeEach, expect, it, vi } from 'vitest'
import { createEditContextMenu } from './context-menu'
import i18n from '@/i18n/config'
const fake = vi.hoisted(() => ({
  resources: [] as Array<{
    close: ReturnType<typeof vi.fn>
    popup: ReturnType<typeof vi.fn>
    setText: ReturnType<typeof vi.fn>
  }>,
}))
vi.mock('@tauri-apps/api/menu', () => {
  const create = async () => {
    const resource = {
      close: vi.fn().mockResolvedValue(undefined),
      popup: vi.fn().mockResolvedValue(undefined),
      setText: vi.fn().mockResolvedValue(undefined),
    }
    fake.resources.push(resource)
    return resource
  }
  return { Menu: { new: create }, MenuItem: { new: create }, PredefinedMenuItem: { new: create } }
})
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({ label: 'main' }) }))
beforeEach(() => {
  fake.resources.length = 0
})
it('retains handles after popup is shown and releases them on owner disposal', async () => {
  const menu = createEditContextMenu()
  await expect(menu.show()).resolves.toBe(true)
  for (const item of fake.resources) expect(item.close).not.toHaveBeenCalled()
  await menu.dispose()
  for (const item of fake.resources) expect(item.close).toHaveBeenCalledOnce()
})
it('cancels late construction without showing a stale menu', async () => {
  const menu = createEditContextMenu()
  await menu.dispose()
  await expect(menu.show()).resolves.toBe(false)
  for (const item of fake.resources) expect(item.popup).not.toHaveBeenCalled()
})

it.each([
  ['en', ['Undo', 'Redo', 'Cut', 'Copy', 'Paste', 'Select All']],
  ['fr', ['Annuler', 'Rétablir', 'Couper', 'Copier', 'Coller', 'Tout sélectionner']],
  ['ar', ['تراجع', 'إعادة', 'قص', 'نسخ', 'لصق', 'تحديد الكل']],
])('shows translated editing labels in %s', async (language, expected) => {
  await i18n.changeLanguage(language)
  const menu = createEditContextMenu()
  try {
    await expect(menu.show()).resolves.toBe(true)
    const labels = fake.resources.flatMap((resource) =>
      resource.setText.mock.calls.map(([text]) => text),
    )
    expect(labels).toEqual(expected)
  } finally {
    await menu.dispose()
    await i18n.changeLanguage('en')
  }
})
