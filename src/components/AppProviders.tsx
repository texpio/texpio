import { QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import type { PropsWithChildren } from 'react'
import { ThemeProvider } from './ThemeProvider'
import { queryClient } from '@/lib/query-client'
import '@/i18n'
export function AppProviders({
  children,
  showDevtools = true,
}: PropsWithChildren<{ showDevtools?: boolean }>) {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>{children}</ThemeProvider>
      {import.meta.env.DEV && showDevtools && <ReactQueryDevtools initialIsOpen={false} />}
    </QueryClientProvider>
  )
}
