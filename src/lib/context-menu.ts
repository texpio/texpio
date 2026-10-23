import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'
import { getCurrentWindow } from '@tauri-apps/api/window'
import i18n from '@/i18n/config'
import { getAllCommands, runCommand, type CommandContext } from './commands'
export type EditRole = 'Undo' | 'Redo' | 'Cut' | 'Copy' | 'Paste' | 'SelectAll' | 'Separator'
export type ContextMenuEntry = { commandId: string } | { role: EditRole }
let generation = 0
/** Retain the owner until its UI unmounts: popup resolves when shown, not dismissed. */
export function createContextMenu(entries: ContextMenuEntry[], context?: CommandContext) {
  let active = true
  let menu: Menu | undefined
  const prefix = `texpio-context-${++generation}-`
  const owned: Array<{ close: () => Promise<void> }> = []
  const labels: Array<{ setText: (text: string) => Promise<void>; key: string }> = []
  const availability: Array<{ item: MenuItem; id: string }> = []
  const own = <T extends { close: () => Promise<void> }>(item: T) => {
    owned.push(item)
    return item
  }
  const close = async () => {
    for (const item of owned.splice(0).reverse()) {
      try {
        await item.close()
      } catch {
        /* Native teardown also releases resources. */
      }
    }
  }
  const ready = Promise.resolve()
    .then(async () => {
      const items: Array<MenuItem | PredefinedMenuItem> = []
      for (const entry of entries) {
        if ('role' in entry) {
          const key = 'menu.' + entry.role[0].toLowerCase() + entry.role.slice(1)
          const item = own(
            await PredefinedMenuItem.new({
              item: entry.role,
              ...(entry.role === 'Separator'
                ? {}
                : { text: i18n.t(key, { defaultValue: entry.role }) }),
            }),
          )
          if (entry.role !== 'Separator')
            labels.push({ setText: (text) => item.setText(text), key })
          items.push(item)
        } else if (context) {
          const command = getAllCommands(context).find((command) => command.id === entry.commandId)
          const item = own(
            await MenuItem.new({
              id: prefix + entry.commandId,
              text: command
                ? i18n.t(command.labelKey, { defaultValue: command.labelKey })
                : i18n.t('commands.error.unavailable'),
              enabled: !!command,
              action: () => {
                if (active) void runCommand(entry.commandId, context)
              },
            }),
          )
          if (command) labels.push({ setText: (text) => item.setText(text), key: command.labelKey })
          availability.push({ item, id: entry.commandId })
          items.push(item)
        }
      }
      menu = own(await Menu.new({ items }))
    })
    .catch(close)
  let disposing: Promise<void> | undefined
  return {
    show: async () => {
      await ready
      if (!active || !menu) return false
      try {
        for (const label of labels)
          await label.setText(i18n.t(label.key, { defaultValue: label.key }))
        if (context)
          for (const { item, id } of availability)
            await item.setEnabled(getAllCommands(context).some((command) => command.id === id))
        if (!active) return false
        await menu.popup(undefined, getCurrentWindow())
        return true
      } catch {
        return false
      }
    },
    dispose: () => {
      active = false
      disposing ??= ready.then(close)
      return disposing
    },
  }
}
export function createEditContextMenu() {
  return createContextMenu(
    ['Undo', 'Redo', 'Separator', 'Cut', 'Copy', 'Paste', 'Separator', 'SelectAll'].map((role) => ({
      role: role as EditRole,
    })),
  )
}
