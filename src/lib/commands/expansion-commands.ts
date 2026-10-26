import type { AppCommand } from './types'

export type ExpansionAction = 'start' | 'pause' | 'stop'
type Controller = {
  can: (action: ExpansionAction) => boolean
  act: (action: ExpansionAction) => void
}
let active: Controller | null = null
export function bindExpansionActions(controller: Controller) {
  active = controller
  return () => {
    if (active === controller) active = null
  }
}
export const expansionCommands: AppCommand[] = (['start', 'pause', 'stop'] as const).map(
  (action) => ({
    id: `expansion-${action}`,
    labelKey: `library.${action}`,
    group: 'expansion',
    palette: false,
    isAvailable: () => active?.can(action) ?? false,
    execute: () => {
      active?.act(action)
    },
  }),
)
