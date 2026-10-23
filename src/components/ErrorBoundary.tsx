import { Component, type PropsWithChildren } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { saveCrashState } from '@/lib/recovery'
import { logger } from '@/lib/logger'
import { safeRelaunch } from '@/lib/updater'
type Props = PropsWithChildren<{ text: (key: string, fallback: string) => string }>
class Boundary extends Component<
  Props,
  { failed: boolean; saved: boolean | null; restarting: boolean; restartFailed: boolean }
> {
  state = { failed: false, saved: null as boolean | null, restarting: false, restartFailed: false }
  private mounted = false
  static getDerivedStateFromError() {
    return { failed: true }
  }
  componentDidMount() {
    this.mounted = true
  }
  componentWillUnmount() {
    this.mounted = false
  }
  componentDidCatch() {
    void logger.error('renderFailed')
    void saveCrashState().then((saved) => {
      if (this.mounted) this.setState({ saved })
    })
  }
  render() {
    if (!this.state.failed) return this.props.children
    const t = this.props.text
    return (
      <main className="mx-auto flex min-h-svh max-w-xl flex-col justify-center gap-5 p-8">
        <h1 className="text-2xl font-semibold">{t('error.title', 'Texpio ran into a problem.')}</h1>
        <p>{t('error.description', 'Restart Texpio to try again.')}</p>
        <p role="status" className="text-sm text-muted-foreground">
          {this.state.saved === null
            ? t('error.saving', 'Saving diagnostics…')
            : this.state.saved
              ? t('error.saved', 'Diagnostic metadata saved.')
              : t('error.notSaved', 'Diagnostics could not be saved.')}
        </p>
        {this.state.restartFailed && (
          <p role="alert">{t('error.restartFailed', 'Restart unavailable. Try again.')}</p>
        )}
        <Button
          disabled={this.state.restarting}
          onClick={() => {
            this.setState({ restarting: true, restartFailed: false })
            void safeRelaunch().then((result) => {
              if (this.mounted) this.setState({ restarting: false, restartFailed: !result.success })
            })
          }}
        >
          {t('error.restart', 'Restart Texpio')}
        </Button>
      </main>
    )
  }
}
export function ErrorBoundary({ children }: PropsWithChildren) {
  const { t } = useTranslation()
  return (
    <Boundary text={(key, fallback) => t(key, { defaultValue: fallback })}>{children}</Boundary>
  )
}
