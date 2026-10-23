import { MainWindow } from '@/components/layout/MainWindow'
import { AppProviders } from '@/components/AppProviders'
import './styles.css'
import { ErrorBoundary } from '@/components/ErrorBoundary'

export default function App() {
  return (
    <ErrorBoundary>
      <AppProviders>
        <MainWindow />
      </AppProviders>
    </ErrorBoundary>
  )
}
