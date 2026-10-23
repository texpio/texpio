import { MainWindow } from '@/components/layout/MainWindow'
import { AppProviders } from '@/components/AppProviders'
import './styles.css'

export default function App() {
  return (
    <AppProviders>
      <MainWindow />
    </AppProviders>
  )
}
