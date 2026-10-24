import { commands, requireDesktop, unwrapResult } from './tauri-bindings'

export async function quitApp() {
  requireDesktop()
  unwrapResult(await commands.quitApp())
}
