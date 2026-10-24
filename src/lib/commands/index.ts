import { registerCommands } from './registry'
import { navigationCommands } from './navigation-commands'
import { windowCommands } from './window-commands'
import { notificationCommands } from './notification-commands'
import { libraryCommands } from './library-commands'
registerCommands([
  ...navigationCommands,
  ...windowCommands,
  ...notificationCommands,
  ...libraryCommands,
])
export {
  registerCommands,
  executeCommand,
  getAllCommands,
  runCommand,
  isAvailable,
} from './registry'
export type { AppCommand, CommandContext, CommandResult } from './types'
