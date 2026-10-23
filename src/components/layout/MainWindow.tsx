import { useEffect, useRef, useState } from 'react'
import type { ImperativePanelHandle } from 'react-resizable-panels'
import { useTranslation } from 'react-i18next'
import { Code, Home, Keyboard } from 'lucide-react'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { TitleBar } from '@/components/titlebar/TitleBar'
import { useUIStore } from '@/store/ui-store'
import { useTheme } from '@/hooks/use-theme'
import { useSquareCornersEffect } from '@/hooks/useSquareCornersEffect'
import { FoundationScreen } from '@/features/foundation/FoundationScreen'
import { LeftSideBar } from './LeftSideBar'
import { RightSideBar } from './RightSideBar'

export function MainWindow() {
  const { t } = useTranslation()
  const layout = useUIStore((state) => state.layout)
  const view = useUIStore((state) => state.activeView)
  const setView = useUIStore((state) => state.setActiveView)
  const sizes = useUIStore((state) => state.setPanelSizes)
  const visible = useUIStore((state) => state.setSidebarVisible)
  const { error } = useTheme()
  const leftRef = useRef<ImperativePanelHandle>(null)
  const rightRef = useRef<ImperativePanelHandle>(null)
  const [layoutReady, setLayoutReady] = useState(false)
  useSquareCornersEffect()
  useEffect(() => {
    if (!layoutReady) return
    for (const [panel, visible, percent] of [
      [leftRef.current, layout.leftVisible, layout.leftPercent],
      [rightRef.current, layout.rightVisible, layout.rightPercent],
    ] as const) {
      if (!panel) continue
      if (visible && Math.abs(panel.getSize() - percent) > 0.1) panel.resize(percent)
      else if (!visible && !panel.isCollapsed()) panel.collapse()
    }
  }, [
    layoutReady,
    layout.leftVisible,
    layout.rightVisible,
    layout.leftPercent,
    layout.rightPercent,
  ])
  return (
    <div className="flex h-svh min-h-0 flex-col overflow-hidden rounded-[var(--app-corner-radius)] bg-background text-foreground">
      <TitleBar />
      {error && (
        <Alert variant="destructive">
          <AlertTitle>{t('shell.preferencesError')}</AlertTitle>
        </Alert>
      )}
      <ResizablePanelGroup
        direction="horizontal"
        className="min-h-0 flex-1"
        onLayout={(values) => {
          if (values.length === 3) {
            sizes(values[0], values[2])
            setLayoutReady(true)
          }
        }}
      >
        <ResizablePanel
          id="left"
          order={1}
          ref={leftRef}
          defaultSize={layout.leftVisible ? layout.leftPercent : 0}
          minSize={15}
          maxSize={40}
          collapsible
          collapsedSize={0}
          onCollapse={() => visible('left', false)}
          onExpand={() => visible('left', true)}
        >
          <LeftSideBar>
            <nav aria-label={t('shell.local')} className="flex flex-col gap-2 p-3">
              <p className="px-2 py-3 text-xs font-semibold text-muted-foreground">
                {t('shell.local')}
              </p>
              <Button
                variant={view === 'home' ? 'secondary' : 'ghost'}
                className="justify-start"
                onClick={() => setView('home')}
              >
                <Home aria-hidden="true" />
                {t('shell.home')}
              </Button>
              <Button
                variant={view === 'developer' ? 'secondary' : 'ghost'}
                className="justify-start"
                onClick={() => setView('developer')}
              >
                <Code aria-hidden="true" />
                {t('shell.developer')}
              </Button>
            </nav>
          </LeftSideBar>
        </ResizablePanel>
        <ResizableHandle disabled={!layout.leftVisible} />
        <ResizablePanel
          id="main"
          order={2}
          defaultSize={
            100 -
            (layout.leftVisible ? layout.leftPercent : 0) -
            (layout.rightVisible ? layout.rightPercent : 0)
          }
          minSize={30}
        >
          <main
            className="h-full min-h-0 overflow-y-auto"
            aria-label={t(view === 'home' ? 'shell.home' : 'shell.developer')}
          >
            {view === 'developer' ? (
              <FoundationScreen />
            ) : (
              <section className="mx-auto flex max-w-2xl flex-col gap-6 px-8 py-16">
                <Keyboard className="size-12 text-muted-foreground" aria-hidden="true" />
                <Badge variant="secondary">{t('shell.ready')}</Badge>
                <h1 className="text-3xl font-semibold tracking-tight">{t('shell.welcome')}</h1>
                <p className="text-muted-foreground">{t('shell.description')}</p>
                <Button className="self-start" onClick={() => setView('developer')}>
                  <Code aria-hidden="true" />
                  {t('shell.openDeveloper')}
                </Button>
              </section>
            )}
          </main>
        </ResizablePanel>
        <ResizableHandle disabled={!layout.rightVisible} />
        <ResizablePanel
          id="right"
          order={3}
          ref={rightRef}
          defaultSize={layout.rightVisible ? layout.rightPercent : 0}
          minSize={15}
          maxSize={40}
          collapsible
          collapsedSize={0}
          onCollapse={() => visible('right', false)}
          onExpand={() => visible('right', true)}
        >
          <RightSideBar>
            <aside aria-label={t('shell.details')} className="flex flex-col gap-4 p-5">
              <h2 className="text-sm font-semibold">{t('shell.details')}</h2>
              <Badge variant="outline">{t('shell.offline')}</Badge>
              <p className="text-sm text-muted-foreground">{t('shell.probeStatus')}</p>
            </aside>
          </RightSideBar>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  )
}
