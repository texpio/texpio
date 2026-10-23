import React from 'react'
import ReactDOM from 'react-dom/client'
import { AppProviders } from '@/components/AppProviders'
import { QuickPaneApp } from '@/components/quick-pane/QuickPaneApp'
import './quick-pane.css'
ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <AppProviders showDevtools={false}>
      <QuickPaneApp />
    </AppProviders>
  </React.StrictMode>,
)
