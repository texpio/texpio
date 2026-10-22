import { useEffect, useState } from 'react'
import { CheckCircle2, LoaderCircle, Monitor, SquareTerminal } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useTheme, type ThemePreference } from '@/features/theme/ThemeProvider'
import { getAppInfo, type AppInfo } from '@/lib/desktop'

type Connection =
  { status: 'loading' } | { status: 'connected'; info: AppInfo } | { status: 'unavailable' }

export function FoundationScreen() {
  const { preference, setPreference } = useTheme()
  const [connection, setConnection] = useState<Connection>({ status: 'loading' })
  const [attempt, setAttempt] = useState(0)

  useEffect(() => {
    let active = true
    getAppInfo().then(
      (info) => {
        if (active) setConnection({ status: 'connected', info })
      },
      () => {
        if (active) setConnection({ status: 'unavailable' })
      },
    )
    return () => {
      active = false
    }
  }, [attempt])

  function retry() {
    setConnection({ status: 'loading' })
    setAttempt((value) => value + 1)
  }

  return (
    <main className="mx-auto flex min-h-svh max-w-3xl flex-col gap-8 px-6 py-9 sm:px-10">
      <header className="flex flex-wrap items-center justify-between gap-5">
        <div className="flex items-center gap-3">
          <div className="flex size-11 items-center justify-center rounded-xl bg-primary text-primary-foreground">
            <SquareTerminal className="size-6" aria-hidden="true" />
          </div>
          <div>
            <h1 className="text-xl font-semibold tracking-tight">Texpio</h1>
            <p className="text-sm text-muted-foreground">Windows desktop foundation</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <label htmlFor="theme" className="text-sm text-muted-foreground">
            Theme
          </label>
          <Select
            value={preference}
            onValueChange={(value) => setPreference(value as ThemePreference)}
          >
            <SelectTrigger id="theme" aria-label="Theme" className="w-28">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">System</SelectItem>
              <SelectItem value="light">Light</SelectItem>
              <SelectItem value="dark">Dark</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </header>
      <Card>
        <CardHeader>
          <CardTitle role="heading" aria-level={2} className="flex items-center gap-2">
            <Monitor className="size-4" aria-hidden="true" />
            Desktop connection
          </CardTitle>
          <CardDescription>Application information from the native desktop shell.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {connection.status === 'loading' && (
            <p role="status" className="flex items-center gap-2 text-sm text-muted-foreground">
              <LoaderCircle
                className="size-4 animate-spin motion-reduce:animate-none"
                aria-hidden="true"
              />
              Connecting to desktop…
            </p>
          )}
          {connection.status === 'connected' && (
            <>
              <p role="status" className="flex items-center gap-2 text-sm">
                <CheckCircle2 className="size-4" aria-hidden="true" />
                Connected
              </p>
              <dl className="grid grid-cols-2 gap-4 border-t pt-5 text-sm">
                <div>
                  <dt className="mb-1 text-muted-foreground">Version</dt>
                  <dd className="font-medium">{connection.info.version}</dd>
                </div>
                <div>
                  <dt className="mb-1 text-muted-foreground">Platform</dt>
                  <dd className="font-medium">{connection.info.platform}</dd>
                </div>
              </dl>
            </>
          )}
          {connection.status === 'unavailable' && (
            <div className="space-y-3">
              <p role="alert" className="text-sm">
                Native desktop information is unavailable.
              </p>
              <p className="text-sm text-muted-foreground">
                Open the Texpio desktop app to connect to its native shell.
              </p>
              <Button variant="outline" size="sm" onClick={retry}>
                Retry
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
      <section aria-labelledby="next-step" className="rounded-xl border border-dashed p-5">
        <h2 id="next-step" className="mb-2 text-sm font-medium">
          Next: native expansion feasibility
        </h2>
        <p className="text-sm leading-relaxed text-muted-foreground">
          Automatic text expansion is not implemented yet. The next milestone will test safe
          abbreviation replacement in Windows applications.
        </p>
      </section>
      <footer className="mt-auto text-xs leading-relaxed text-muted-foreground">
        Closing this window keeps Texpio running. To exit, choose Quit from the tray menu.
      </footer>
    </main>
  )
}
