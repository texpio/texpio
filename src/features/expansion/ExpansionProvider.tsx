import { useLayoutEffect, type ReactNode } from 'react'
import { useExpansionController } from '@/services/expansion'
import { useUIStore } from '@/store/ui-store'
import { bindExpansionActions } from '@/lib/commands/expansion-commands'
import { ExpansionContext } from './use-expansion'

export function ExpansionProvider({ children }: { children: ReactNode }) {
  const blocked = useUIStore((state) => state.expansionStartBlocked)
  const controller = useExpansionController(blocked)
  useLayoutEffect(() => bindExpansionActions(controller))
  return <ExpansionContext.Provider value={controller}>{children}</ExpansionContext.Provider>
}
