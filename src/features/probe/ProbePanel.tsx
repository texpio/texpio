import { useCallback, useEffect, useRef, useState } from 'react'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Field, FieldDescription, FieldGroup, FieldLabel } from '@/components/ui/field'
import { NativeSelect, NativeSelectOption } from '@/components/ui/native-select'
import { Separator } from '@/components/ui/separator'
import {
  excludeProbe,
  getProbeFixtures,
  getProbeStatus,
  pauseProbe,
  ProbeCommandError,
  profiles,
  refusalLabels,
  startProbe,
  stopProbe,
  unavailableMessage,
  type InsertionStrategy,
  type ProbeFixture,
  type ProbeStatus,
  type ReferenceProfile,
} from '@/lib/probe'

const initialStatus: ProbeStatus = {
  phase: 'stopped',
  profile: null,
  strategy: 'unicode',
  counters: { submitted: 0, observed: 0, refused: 0, uncertain: 0 },
  refusalReason: null,
  lastActivityAgeMs: null,
}
const phaseLabels = { stopped: 'Stopped', running: 'Running', paused: 'Paused', fault: 'Fault' }
type Pending = 'start' | 'stop' | 'pause' | 'exclude' | null

export function ProbePanel() {
  const [status, setStatus] = useState(initialStatus)
  const [fixtures, setFixtures] = useState<ProbeFixture[]>([])
  const [profile, setProfile] = useState<ReferenceProfile | ''>('')
  const [strategy, setStrategy] = useState<InsertionStrategy>('unicode')
  const [available, setAvailable] = useState(false)
  const [connectionError, setConnectionError] = useState(false)
  const [commandError, setCommandError] = useState<string | null>(null)
  const [pending, setPending] = useState<Pending>(null)
  const [attempt, setAttempt] = useState(0)
  const pendingRef = useRef<Pending>(null)
  const revision = useRef(0)
  const nextRevision = useCallback(() => ++revision.current, [])
  const mounted = useRef(false)

  function applyStatus(next: ProbeStatus) {
    setStatus(next)
    setProfile((selected) =>
      next.phase === 'running' || next.phase === 'paused'
        ? (next.profile ?? selected)
        : selected || next.profile || '',
    )
    if (next.phase === 'running' || next.phase === 'paused') setStrategy(next.strategy)
  }

  useEffect(() => {
    mounted.current = true
    const initialRevision = nextRevision()
    let active = true
    let initialized = false
    Promise.all([getProbeStatus(), getProbeFixtures()]).then(
      ([next, items]) => {
        if (!active || revision.current !== initialRevision) return
        initialized = true
        applyStatus(next)
        setFixtures(items)
        setAvailable(true)
        setConnectionError(false)
      },
      () => {
        if (active && revision.current === initialRevision) {
          setAvailable(false)
          setConnectionError(true)
        }
      },
    )
    const timer = setInterval(() => {
      if (!active || pendingRef.current || !initialized) return
      const requestRevision = revision.current
      getProbeStatus().then(
        (next) => {
          if (active && requestRevision === revision.current && !pendingRef.current) {
            applyStatus(next)
            setAvailable(true)
            setConnectionError(false)
          }
        },
        () => {
          if (active && requestRevision === revision.current && !pendingRef.current) {
            setAvailable(false)
            setConnectionError(true)
          }
        },
      )
    }, 500)
    return () => {
      active = false
      mounted.current = false
      nextRevision()
      clearInterval(timer)
    }
  }, [attempt, nextRevision])

  async function mutate(kind: Exclude<Pending, null>, operation: () => Promise<ProbeStatus>) {
    if (pendingRef.current && (kind !== 'stop' || pendingRef.current === 'stop')) return
    const current = nextRevision()
    pendingRef.current = kind
    setPending(kind)
    setCommandError(null)
    try {
      const next = await operation()
      if (mounted.current && current === revision.current) applyStatus(next)
    } catch (error) {
      if (mounted.current && current === revision.current) {
        setCommandError(
          error instanceof ProbeCommandError ? refusalLabels[error.code] : unavailableMessage,
        )
      }
    } finally {
      if (mounted.current && current === revision.current) {
        pendingRef.current = null
        setPending(null)
      }
    }
  }

  const active = status.phase === 'running' || status.phase === 'paused'
  const error = connectionError ? unavailableMessage : commandError
  const labels = {
    submitted: 'Submitted requests',
    observed: 'Observed expansions',
    refused: 'Refused',
    uncertain: 'Uncertain',
  }
  return (
    <Card aria-labelledby="probe-heading">
      <CardHeader>
        <CardTitle id="probe-heading" role="heading" aria-level={2}>
          Windows expansion probe
        </CardTitle>
        <CardDescription>
          Experimental feasibility controls. Use disposable test documents; the engine starts
          stopped on every app launch.
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-6">
        <p role="status" aria-label="Expansion status">
          {pending === 'start'
            ? 'Starting…'
            : pending === 'stop'
              ? 'Stopping…'
              : phaseLabels[status.phase]}
        </p>
        {error && (
          <Alert variant="destructive">
            <AlertTitle>Probe notice</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}
        {connectionError && (
          <Button
            variant="outline"
            onClick={() => {
              setAvailable(false)
              setCommandError(null)
              setAttempt((value) => value + 1)
            }}
          >
            Retry probe connection
          </Button>
        )}
        <FieldGroup>
          <Field data-disabled={!available || !!pending || active}>
            <FieldLabel htmlFor="probe-profile">Reference app</FieldLabel>
            <NativeSelect
              id="probe-profile"
              value={profile}
              disabled={!available || !!pending || active}
              onChange={(event) =>
                setProfile(profiles.find((item) => item === event.target.value) ?? '')
              }
            >
              <NativeSelectOption value="">Select a reference app</NativeSelectOption>
              <NativeSelectOption value="notepad">Notepad test document</NativeSelectOption>
              <NativeSelectOption value="vsCode">
                VS Code plaintext test document
              </NativeSelectOption>
              <NativeSelectOption value="chromeFixture">Chrome loopback fixture</NativeSelectOption>
            </NativeSelect>
            <FieldDescription>
              Only the selected reference control can expand. Other applications and password fields
              are refused.
            </FieldDescription>
          </Field>
          <Field data-disabled={!available || !!pending || active}>
            <FieldLabel htmlFor="probe-strategy">Insertion strategy</FieldLabel>
            <NativeSelect
              id="probe-strategy"
              value={strategy}
              disabled={!available || !!pending || active}
              onChange={(event) => {
                if (event.target.value === 'unicode') setStrategy('unicode')
              }}
            >
              <NativeSelectOption value="unicode">Direct Unicode</NativeSelectOption>
              <NativeSelectOption value="clipboard" disabled>
                Clipboard — qualification pending
              </NativeSelectOption>
            </NativeSelect>
            <FieldDescription>{refusalLabels.clipboardState}</FieldDescription>
          </Field>
        </FieldGroup>
        {status.refusalReason && (
          <p className="text-sm text-muted-foreground" aria-live="polite">
            {refusalLabels[status.refusalReason]}
          </p>
        )}
        <dl className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {(Object.keys(labels) as Array<keyof typeof labels>).map((key) => (
            <div key={key} data-testid={`counter-${key}`}>
              <dt className="text-sm text-muted-foreground">{labels[key]}</dt>
              <dd className="text-lg font-medium tabular-nums">{status.counters[key]}</dd>
            </div>
          ))}
        </dl>
        <p className="text-sm text-muted-foreground">
          Submitted requests are counted separately from observed output. The tray can pause or stop
          the probe while this window is hidden.
        </p>
        <Separator />
        <section aria-labelledby="probe-fixtures">
          <h3 id="probe-fixtures" className="mb-3 text-sm font-medium">
            Fixed test snippets
          </h3>
          <div className="flex flex-col gap-3">
            {fixtures.map((fixture) => (
              <details key={fixture.id}>
                <summary className="cursor-pointer text-sm">
                  {fixture.title} <code>{fixture.trigger}</code>
                </summary>
                <pre className="mt-2 max-h-40 overflow-auto whitespace-pre-wrap break-words rounded-md bg-muted p-3 text-sm">
                  {fixture.body}
                </pre>
              </details>
            ))}
          </div>
          <p className="mt-3 text-sm text-muted-foreground">
            Type a shortcut followed by Space. One trailing space remains. Start at the beginning of
            the field or after whitespace.
          </p>
        </section>
      </CardContent>
      <CardFooter className="flex flex-wrap gap-2">
        <Button
          disabled={!available || !profile || !!pending || active}
          onClick={() => profile && void mutate('start', () => startProbe({ profile, strategy }))}
        >
          Start probe
        </Button>
        <Button
          variant="outline"
          disabled={!available || !!pending || !active}
          onClick={() => void mutate('pause', () => pauseProbe(status.phase !== 'paused'))}
        >
          {status.phase === 'paused' ? 'Resume' : 'Pause'}
        </Button>
        <Button
          variant="outline"
          disabled={pending === 'stop' || (!available && !pending && status.phase === 'stopped')}
          onClick={() => void mutate('stop', stopProbe)}
        >
          Stop
        </Button>
        <Button
          variant="ghost"
          disabled={!available || !profile || !!pending}
          onClick={() => profile && void mutate('exclude', () => excludeProbe(profile))}
        >
          Exclude this app
        </Button>
      </CardFooter>
    </Card>
  )
}
