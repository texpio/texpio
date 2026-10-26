import { createContext, useContext } from 'react'
import type { useExpansionController } from '@/services/expansion'
import type { ExpansionAction } from '@/lib/commands/expansion-commands'
import { runCommand } from '@/lib/commands'
import { useCommandContext } from '@/hooks/use-command-context'

export const ExpansionContext = createContext<ReturnType<typeof useExpansionController> | null>(
  null,
)
export function useExpansion() {
  const controller = useContext(ExpansionContext)
  const context = useCommandContext()
  if (!controller) throw new Error('ExpansionProvider is required.')
  return {
    ...controller,
    act: (action: ExpansionAction) => {
      void runCommand(`expansion-${action}`, context)
    },
  }
}
