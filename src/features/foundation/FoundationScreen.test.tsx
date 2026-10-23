import { invoke, isTauri } from '@tauri-apps/api/core'
import { act, render, screen } from '@testing-library/react'
import { StrictMode } from 'react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { ThemeProvider } from '@/features/theme/ThemeProvider'
import { mockSystemTheme } from '@/test/system-theme'
import { FoundationScreen } from './FoundationScreen'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), isTauri: vi.fn() }))
vi.mock('@/features/probe/ProbePanel', () => ({
  ProbePanel: () => <section aria-label="Windows expansion probe" />,
}))

beforeEach(() => {
  mockSystemTheme()
  vi.mocked(isTauri).mockReturnValue(true)
})

function renderScreen() {
  return render(
    <ThemeProvider>
      <FoundationScreen />
    </ThemeProvider>,
  )
}

describe('foundation screen', () => {
  it('shows loading until native metadata arrives', () => {
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}))
    renderScreen()
    expect(screen.getByRole('status')).toHaveTextContent(/connecting/i)
    expect(screen.queryByText('0.1.0')).not.toBeInTheDocument()
  })

  it('shows real bridge data and the current scope', async () => {
    vi.mocked(invoke).mockResolvedValue({ version: '0.1.0', platform: 'windows' })
    renderScreen()
    expect(await screen.findByText('0.1.0')).toBeInTheDocument()
    expect(screen.getByText('windows')).toBeInTheDocument()
    expect(screen.getByRole('status')).toHaveTextContent(/connected/i)
    expect(screen.getByRole('combobox', { name: /theme/i })).toBeInTheDocument()
    expect(screen.getByLabelText('Windows expansion probe')).toBeInTheDocument()
    expect(screen.getByText(/closing this window keeps texpio running/i)).toBeInTheDocument()
  })

  it('recovers from a native failure with Retry', async () => {
    vi.mocked(invoke)
      .mockRejectedValueOnce(new Error('C:/private/example'))
      .mockResolvedValueOnce({ version: '0.1.0', platform: 'windows' })
    renderScreen()
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Native desktop information is unavailable.',
    )
    expect(screen.queryByText(/private/)).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: /retry/i }))
    expect(await screen.findByText('0.1.0')).toBeInTheDocument()
  })

  it('does not invent a desktop connection in a browser preview', async () => {
    vi.mocked(isTauri).mockReturnValue(false)
    renderScreen()
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Native desktop information is unavailable.',
    )
    expect(screen.queryByText('0.1.0')).not.toBeInTheDocument()
    expect(invoke).not.toHaveBeenCalled()
  })

  it('treats a malformed native response as unavailable', async () => {
    vi.mocked(invoke).mockResolvedValue({ version: null, platform: 'windows' })
    renderScreen()
    expect(await screen.findByRole('alert')).toBeInTheDocument()
  })

  it('keeps a stale request from replacing a newer connection', async () => {
    let finish!: (value: unknown) => void
    vi.mocked(invoke)
      .mockReturnValueOnce(
        new Promise((resolve) => {
          finish = resolve
        }),
      )
      .mockResolvedValueOnce({ version: '0.2.0', platform: 'windows' })
    render(
      <StrictMode>
        <ThemeProvider>
          <FoundationScreen />
        </ThemeProvider>
      </StrictMode>,
    )
    expect(await screen.findByText('0.2.0')).toBeInTheDocument()
    await act(async () => finish({ version: '0.1.0', platform: 'windows' }))
    expect(screen.getByText('0.2.0')).toBeInTheDocument()
    expect(screen.queryByText('0.1.0')).not.toBeInTheDocument()
  })
})
