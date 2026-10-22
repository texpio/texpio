import { vi } from 'vitest'

export function mockSystemTheme(initialDark = false) {
  let dark = initialDark
  const listeners = new Set<EventListenerOrEventListenerObject>()
  const query = {
    get matches() {
      return dark
    },
    media: '(prefers-color-scheme: dark)',
    onchange: null,
    addEventListener: (_type: string, listener: EventListenerOrEventListenerObject) =>
      listeners.add(listener),
    removeEventListener: (_type: string, listener: EventListenerOrEventListenerObject) =>
      listeners.delete(listener),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: () => true,
  } satisfies MediaQueryList
  vi.stubGlobal(
    'matchMedia',
    vi.fn(() => query),
  )
  return (nextDark: boolean) => {
    dark = nextDark
    const event = { matches: dark, media: query.media } as MediaQueryListEvent
    listeners.forEach((listener) => {
      if (typeof listener === 'function') listener(event)
      else listener.handleEvent(event)
    })
  }
}
