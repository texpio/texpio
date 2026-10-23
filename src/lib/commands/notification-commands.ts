import i18n from '@/i18n/config'
import { useUIStore } from '@/store/ui-store'
import type { AppCommand } from './types'
export const notificationCommands: AppCommand[] = [
  {
    id: 'test-toast',
    labelKey: 'commands.testToast.label',
    descriptionKey: 'commands.testToast.description',
    group: 'developer',
    isAvailable: () => useUIStore.getState().activeView === 'developer',
    execute: (context) => context.showToast(i18n.t('commands.testToast.label'), 'info'),
  },
]
