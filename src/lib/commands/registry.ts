import i18n from '@/i18n/config'
import type { AppCommand, CommandContext, CommandResult } from './types'
const registry = new Map<string, AppCommand>()
export function registerCommands(commands: AppCommand[]) {
  for (const command of commands) registry.set(command.id, command)
}
export function isAvailable(command: AppCommand, context: CommandContext) {
  try {
    return !command.isAvailable || command.isAvailable(context)
  } catch {
    return false
  }
}
const normalize = (text: string) =>
  text
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLocaleLowerCase()
export function getAllCommands(
  context: CommandContext,
  search = '',
  translate: (key: string) => string = (key) => i18n.t(key, { defaultValue: key }),
) {
  const query = normalize(search.trim())
  return [...registry.values()]
    .filter((command) => isAvailable(command, context))
    .filter(
      (command) =>
        !query ||
        normalize(
          [
            translate(command.labelKey),
            command.descriptionKey ? translate(command.descriptionKey) : '',
            ...(command.keywords ?? []),
          ].join(' '),
        ).includes(query),
    )
}
export async function executeCommand(id: string, context: CommandContext): Promise<CommandResult> {
  try {
    const command = registry.get(id)
    if (!command) return { success: false, code: 'unknownCommand' }
    if (!isAvailable(command, context)) return { success: false, code: 'unavailable' }
    await command.execute(context)
    return { success: true }
  } catch {
    return { success: false, code: 'failed' }
  }
}
export async function runCommand(id: string, context: CommandContext) {
  const result = await executeCommand(id, context)
  if (!result.success) {
    try {
      context.showToast(
        i18n.t(`commands.error.${result.code}`, { defaultValue: 'This command is unavailable.' }),
        'error',
      )
    } catch {
      /* Feedback cannot reopen the failure boundary. */
    }
  }
  return result
}
