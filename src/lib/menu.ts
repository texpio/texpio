import { Menu, MenuItem, Submenu, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { getCurrentWindow } from '@tauri-apps/api/window'
import i18n from '@/i18n/config'
import { runCommand, type CommandContext } from './commands'
export { formatShortcut } from './shortcut-format'

export function createMenuAction(id: string, context: CommandContext) {
  return () => runCommand(id, context)
}
let generation = 0
type Resource = { close: () => Promise<void> }
type LabelTarget = { setText: (text: string) => Promise<void> }
export function startNativeMenu(context: CommandContext) {
  let active = context.platform !== 'unknown'
  const prefix = `texpio-menu-${++generation}-`
  const owned: Resource[] = []
  const labels: Array<{ item: LabelTarget; key: string }> = []
  const own = <T extends Resource>(item: T) => {
    owned.push(item)
    return item
  }
  const text = (key: string) => i18n.t(key, { defaultValue: key, appName: 'Texpio' })
  const label = <T extends LabelTarget>(item: T, key: string) => {
    labels.push({ item, key })
    return item
  }
  const close = async () => {
    for (const item of owned.splice(0).reverse()) {
      try {
        await item.close()
      } catch {
        /* Native window teardown also releases its resource table. */
      }
    }
  }
  const command = async (id: string, key: string) =>
    own(
      label(
        await MenuItem.new({
          id: prefix + id,
          text: text(key),
          action: () => {
            if (active) void createMenuAction(id, context)()
          },
        }),
        key,
      ),
    )
  const build = async () => {
    if (!active) return false
    const about = own(
      label(
        await PredefinedMenuItem.new({
          item: { About: { name: 'Texpio', version: __APP_VERSION__ } },
          text: text('menu.about'),
        }),
        'menu.about',
      ),
    )
    const preferences = await command('open-preferences', 'menu.preferences')
    const updates = await command('check-for-updates', 'menu.checkForUpdates')
    const appItems: Array<MenuItem | PredefinedMenuItem> = [about, preferences, updates]
    if (context.platform === 'macos') {
      for (const [item, key] of [
        ['Hide', 'menu.hide'],
        ['HideOthers', 'menu.hideOthers'],
        ['ShowAll', 'menu.showAll'],
        ['Quit', 'menu.quit'],
      ] as const)
        appItems.push(own(label(await PredefinedMenuItem.new({ item, text: text(key) }), key)))
    }
    const appMenu = own(await Submenu.new({ id: prefix + 'app', text: 'Texpio', items: appItems }))
    const viewItems = []
    for (const [id, key] of [
      ['show-quick-pane', 'quickPane.open'],
      ['open-home', 'shell.home'],
      ['open-developer', 'shell.developer'],
      ['toggle-command-palette', 'commandPalette.title'],
      ['toggle-left-sidebar', 'menu.toggleLeftSidebar'],
      ['toggle-right-sidebar', 'menu.toggleRightSidebar'],
    ] as const)
      viewItems.push(await command(id, key))
    const viewMenu = own(
      label(
        await Submenu.new({ id: prefix + 'view', text: text('menu.view'), items: viewItems }),
        'menu.view',
      ),
    )
    const editItems = []
    for (const [item, key] of [
      ['Undo', 'menu.undo'],
      ['Redo', 'menu.redo'],
      ['Cut', 'menu.cut'],
      ['Copy', 'menu.copy'],
      ['Paste', 'menu.paste'],
      ['SelectAll', 'menu.selectAll'],
    ] as const)
      editItems.push(own(label(await PredefinedMenuItem.new({ item, text: text(key) }), key)))
    const editMenu = own(
      label(
        await Submenu.new({ id: prefix + 'edit', text: text('menu.edit'), items: editItems }),
        'menu.edit',
      ),
    )
    const menu = own(await Menu.new({ id: prefix + 'root', items: [appMenu, editMenu, viewMenu] }))
    if (!active) return false
    // App-specific chords are owned by the DOM router. Native editing roles keep OS ownership.
    const previous =
      context.platform === 'macos'
        ? await menu.setAsAppMenu()
        : await menu.setAsWindowMenu(getCurrentWindow())
    if (previous) {
      try {
        await previous.close()
      } catch {
        /* The prior owner's resources are disposed independently. */
      }
    }
    return true
  }
  const ready = Promise.resolve()
    .then(build)
    .catch(async () => {
      await close()
      return false
    })
  let sequence: Promise<unknown> = ready
  const changed = () => {
    sequence = sequence
      .then(async () => {
        if (active) for (const { item, key } of labels) await item.setText(text(key))
      })
      .catch(() => {})
  }
  if (active) i18n.on('languageChanged', changed)
  let disposing: Promise<void> | undefined
  return {
    ready,
    flush: () => sequence.then(() => {}),
    dispose: () => {
      active = false
      i18n.off('languageChanged', changed)
      disposing ??= sequence.then(close)
      return disposing
    },
  }
}
