import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event'
import { dismissQuickPane } from '@/services/quick-pane'
export function QuickPaneApp() {
  const { t } = useTranslation()
  const [text, setText] = useState('')
  const [failed, setFailed] = useState(false)
  const input = useRef<HTMLInputElement>(null)
  const busy = useRef(false)
  useEffect(() => {
    let cancelled = false
    let stop: (() => void) | undefined
    void getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (cancelled) return
        if (focused) {
          setFailed(false)
          input.current?.focus()
        } else {
          setText('')
          if (!busy.current)
            void dismissQuickPane().catch(() => {
              if (!cancelled) setFailed(true)
            })
        }
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
      stop?.()
    }
  }, [])
  const dismiss = async () => {
    if (busy.current) return
    busy.current = true
    setText('')
    try {
      await dismissQuickPane()
    } catch {
      setFailed(true)
    } finally {
      busy.current = false
    }
  }
  return (
    <form
      className="flex h-svh w-full items-center gap-3 rounded-[var(--app-corner-radius)] border bg-background px-5 text-foreground shadow-lg"
      onSubmit={(event) => {
        event.preventDefault()
        if (busy.current) return
        const entry = text.trim()
        if (!entry) {
          void dismiss()
          return
        }
        busy.current = true
        setFailed(false)
        setText('')
        void (async () => {
          try {
            await emit('quick-pane-submit', { text: entry })
          } catch {
            setText(entry)
            setFailed(true)
            return
          }
          try {
            await dismissQuickPane()
          } catch {
            setFailed(true)
          }
        })().finally(() => {
          busy.current = false
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
        onChange={(event) => setText(event.target.value)}
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
