import { useMemo } from 'react'
import { toast } from 'sonner'
import { usePlatform } from './use-platform'
import { useUIStore } from '@/store/ui-store'
import { executeWindowAction } from '@/lib/window-actions'
import type { CommandContext } from '@/lib/commands'
export function useCommandContext(): CommandContext {
  const platform = usePlatform()
  return useMemo(
    () => ({
      platform,
      openPreferences: () => useUIStore.getState().setActiveView('preferences'),
      showToast: (message, type = 'info') => {
        toast[type](message)
      },
      windowAction: executeWindowAction,
    }),
    [platform],
  )
}
