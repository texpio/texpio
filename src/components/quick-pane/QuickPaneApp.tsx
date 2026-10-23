import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event'
import { dismissQuickPane } from '@/services/quick-pane'
export function QuickPaneApp() {
  const { t } = useTranslation()
  const [text, setText] = useState('')
  const [failed, setFailed] = useState(false)
  const input = useRef<HTMLInputElement>(null)
  const generation = useRef(0)
  const busy = useRef<number | null>(null)
  const mounted = useRef(true)
  const nextInteraction = useCallback(() => {
    busy.current = null
    return ++generation.current
  }, [])
  const dismiss = useCallback(async () => {
    const owner = nextInteraction()
    setText('')
    setFailed(false)
    try {
      await dismissQuickPane()
    } catch {
      if (mounted.current && generation.current === owner) setFailed(true)
    }
  }, [nextInteraction])
  useEffect(() => {
    mounted.current = true
    let cancelled = false
    let stop: (() => void) | undefined
    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (cancelled) return
        if (focused) {
          nextInteraction()
          setFailed(false)
          input.current?.focus()
        } else void dismiss()
      })
      .then(
        (unlisten) => {
          if (cancelled) unlisten()
          else stop = unlisten
        },
        () => {
          if (!cancelled) setFailed(true)
        },
      )
    return () => {
      cancelled = true
      mounted.current = false
      nextInteraction()
      stop?.()
    }
  }, [dismiss, nextInteraction])
  return (
    <form
      className="flex h-svh w-full items-center gap-3 rounded-[var(--app-corner-radius)] border bg-background px-5 text-foreground shadow-lg"
      onSubmit={(event) => {
        event.preventDefault()
        if (busy.current !== null) return
        const entry = text.trim()
        if (!entry) {
          void dismiss()
          return
        }
        const owner = nextInteraction()
        busy.current = owner
        setFailed(false)
        setText('')
        void (async () => {
          try {
            await emit('quick-pane-submit', { text: entry })
          } catch {
            if (mounted.current && generation.current === owner) {
              setText(entry)
              setFailed(true)
            }
            return
          }
          if (!mounted.current || generation.current !== owner) return
          try {
            await dismissQuickPane()
          } catch {
            if (mounted.current && generation.current === owner) setFailed(true)
          }
        })().finally(() => {
          if (busy.current === owner) busy.current = null
        })
      }}
    >
      <input
        ref={input}
        type="text"
        value={text}
        maxLength={16384}
        aria-label={t('quickPane.placeholder')}
        placeholder={t('quickPane.placeholder')}
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
        spellCheck={false}
        className="w-full bg-transparent text-lg outline-none placeholder:text-muted-foreground"
        onChange={(event) => {
          nextInteraction()
          setText(event.target.value)
        }}
        onKeyDown={(event) => {
          if (event.key === 'Escape') {
            event.preventDefault()
            void dismiss()
          } else if (event.key === 'Enter' && (event.repeat || event.nativeEvent.isComposing))
            event.preventDefault()
        }}
      />
      {failed && (
        <span role="alert" className="text-xs text-destructive">
          {t('quickPane.unavailable')}
        </span>
      )}
    </form>
  )
}
