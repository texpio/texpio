import { registerCommands } from './registry'
import { navigationCommands } from './navigation-commands'
import { windowCommands } from './window-commands'
import { libraryCommands } from './library-commands'
registerCommands([...navigationCommands, ...windowCommands, ...libraryCommands])
export {
  registerCommands,
  executeCommand,
  getAllCommands,
  runCommand,
  isAvailable,
} from './registry'
export type { AppCommand, CommandContext, CommandResult } from './types'
