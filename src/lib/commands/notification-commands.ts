import i18n from '@/i18n/config'
import { useUIStore } from '@/store/ui-store'
import type { AppCommand } from './types'
import { notify } from '@/lib/notifications'
export const notificationCommands: AppCommand[] = [
  {
    id: 'test-native-notification',
    labelKey: 'commands.testNativeNotification.label',
    group: 'developer',
    isAvailable: (context) =>
      context.platform !== 'unknown' && useUIStore.getState().activeView === 'developer',
    execute: async () => {
      await notify('Texpio', i18n.t('commands.testNativeNotification.label'), { native: true })
    },
  },
  {
    id: 'test-toast',
    labelKey: 'commands.testToast.label',
    descriptionKey: 'commands.testToast.description',
    group: 'developer',
    isAvailable: () => useUIStore.getState().activeView === 'developer',
    execute: (context) => context.showToast(i18n.t('commands.testToast.label'), 'info'),
  },
]
