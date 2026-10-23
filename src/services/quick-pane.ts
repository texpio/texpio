import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
export async function showQuickPane() {
  requireDesktop()
  unwrapResult(await commands.showQuickPane())
}
export async function dismissQuickPane() {
  requireDesktop()
  unwrapResult(await commands.dismissQuickPane())
}
export async function updateQuickPaneShortcut(shortcut: string) {
  requireDesktop()
  unwrapResult(await commands.updateQuickPaneShortcut(shortcut))
}
export async function getDefaultQuickPaneShortcut() {
  requireDesktop()
  const value = await commands.getDefaultQuickPaneShortcut()
  if (typeof value !== 'string' || !/^[\x20-\x7e]{1,128}$/.test(value))
    throw new Error('Shortcut unavailable.')
  return value
}
