import { FoundationScreen } from '@/features/foundation/FoundationScreen'
import { ThemeProvider } from '@/features/theme/ThemeProvider'
import './styles.css'

export default function App() {
  return (
    <ThemeProvider>
      <FoundationScreen />
    </ThemeProvider>
  )
}
