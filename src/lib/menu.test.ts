import { beforeEach, expect, it, vi } from 'vitest'
import i18n from '@/i18n/config'
import { startNativeMenu } from './menu'
import type { CommandContext } from './commands'

const fake = vi.hoisted(() => ({
  resources: [] as Array<{
    close: ReturnType<typeof vi.fn>
    setText: ReturnType<typeof vi.fn>
    options: Record<string, unknown>
  }>,
  attach: vi.fn(),
}))
vi.mock('@tauri-apps/api/menu', () => {
  const create = async (options: Record<string, unknown>) => {
    const item = {
      options,
      close: vi.fn().mockResolvedValue(undefined),
      setText: vi.fn().mockResolvedValue(undefined),
      setAsAppMenu: fake.attach,
      setAsWindowMenu: fake.attach,
    }
    fake.resources.push(item)
    return item
  }
  return {
    Menu: { new: create },
    MenuItem: { new: create },
    Submenu: { new: create },
    PredefinedMenuItem: { new: create },
  }
})
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({ label: 'main' }) }))
const context: CommandContext = {
  platform: 'windows',
  openPreferences: vi.fn(),
  showToast: vi.fn(),
  windowAction: vi.fn(),
}
beforeEach(() => {
  fake.resources.length = 0
  fake.attach.mockResolvedValue(null)
})
it('updates labels without replacing handlers and disposes every owned handle', async () => {
  const first = startNativeMenu(context)
  expect(await first.ready).toBe(true)
  const count = fake.resources.length
  const ids = fake.resources.map((item) => item.options.id).filter(Boolean)
  await i18n.changeLanguage('fr')
  await first.flush()
  expect(fake.resources).toHaveLength(count)
  expect(fake.resources.some((item) => item.setText.mock.calls.length > 0)).toBe(true)
  await first.dispose()
  for (const item of fake.resources) expect(item.close).toHaveBeenCalledOnce()
  const calls = fake.resources.reduce((sum, item) => sum + item.setText.mock.calls.length, 0)
  await i18n.changeLanguage('ar')
  await first.flush()
  expect(fake.resources.reduce((sum, item) => sum + item.setText.mock.calls.length, 0)).toBe(calls)
  const second = startNativeMenu(context)
  await second.ready
  for (const item of fake.resources.slice(count))
    if (item.options.id) expect(ids).not.toContain(item.options.id)
  await second.dispose()
})
it('cancels startup without attaching a late menu', async () => {
  const menu = startNativeMenu(context)
  await menu.dispose()
  expect(fake.attach).not.toHaveBeenCalled()
})
