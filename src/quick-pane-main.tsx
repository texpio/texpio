import React from 'react'
import ReactDOM from 'react-dom/client'
import { AppProviders } from '@/components/AppProviders'
import { QuickPaneApp } from '@/components/quick-pane/QuickPaneApp'
import './quick-pane.css'
import { logger } from '@/lib/logger'
ReactDOM.createRoot(document.getElementById('root') as HTMLElement, {
  onCaughtError: () => {
    void logger.error('renderFailed')
  },
  onUncaughtError: () => {
    void logger.error('renderFailed')
  },
  onRecoverableError: () => {
    void logger.warn('renderFailed')
  },
}).render(
  <React.StrictMode>
    <AppProviders showDevtools={false}>
      <QuickPaneApp />
    </AppProviders>
  </React.StrictMode>,
)
