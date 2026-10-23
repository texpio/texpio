import { beforeEach, expect, it, vi } from 'vitest'
import { createEditContextMenu } from './context-menu'
const fake = vi.hoisted(() => ({
  resources: [] as Array<{ close: ReturnType<typeof vi.fn>; popup: ReturnType<typeof vi.fn> }>,
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
