import { registerCommands } from './registry'
import { navigationCommands } from './navigation-commands'
import { windowCommands } from './window-commands'
import { libraryCommands } from './library-commands'
import { expansionCommands } from './expansion-commands'
registerCommands([
  ...navigationCommands,
  ...windowCommands,
  ...libraryCommands,
  ...expansionCommands,
])
export {
  registerCommands,
  executeCommand,
  getAllCommands,
  runCommand,
  isAvailable,
} from './registry'
export type { AppCommand, CommandContext, CommandResult } from './types'
