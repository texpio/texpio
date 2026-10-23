import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
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
    <App />
  </React.StrictMode>,
)
