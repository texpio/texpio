import { useEffect, useRef, useState } from 'react'
import type { ImperativePanelHandle } from 'react-resizable-panels'
import { useTranslation } from 'react-i18next'
import { ResizablePanelGroup, ResizablePanel, ResizableHandle } from '@/components/ui/resizable'
import { Alert, AlertTitle } from '@/components/ui/alert'
import { TitleBar } from '@/components/titlebar/TitleBar'
import { useUIStore } from '@/store/ui-store'
import { useTheme } from '@/hooks/use-theme'
import { useSquareCornersEffect } from '@/hooks/useSquareCornersEffect'
import { LibraryView } from '@/features/library/LibraryView'
import { LibraryNavigation } from '@/features/library/LibraryNavigation'
import { CommandPalette } from '@/components/command-palette/CommandPalette'
import { PreferencesPage } from '@/components/preferences/PreferencesPage'
import { Toaster } from '@/components/ui/sonner'
import { UpdateDialog } from '@/components/updates/UpdateDialog'
import { useMainWindowEventListeners } from '@/hooks/useMainWindowEventListeners'
import { LeftSideBar } from './LeftSideBar'
import { ExpansionProvider } from '@/features/expansion/ExpansionProvider'
import { ExpansionNotice } from '@/features/expansion/ExpansionNotice'

export function MainWindow() {
  return (
    <ExpansionProvider>
      <MainWindowLayout />
    </ExpansionProvider>
  )
}
function MainWindowLayout() {
  const { t } = useTranslation()
  const layout = useUIStore((state) => state.layout)
  const view = useUIStore((state) => state.activeView)
  const sizes = useUIStore((state) => state.setSidebarWidth)
  const visible = useUIStore((state) => state.setSidebarVisible)
  const { error } = useTheme()
  const leftRef = useRef<ImperativePanelHandle>(null)
  const [layoutReady, setLayoutReady] = useState(false)
  useSquareCornersEffect()
  useMainWindowEventListeners()
  useEffect(() => {
    const panel = leftRef.current
    if (!layoutReady || !panel) return
    if (layout.leftVisible && Math.abs(panel.getSize() - layout.leftPercent) > 0.1)
      panel.resize(layout.leftPercent)
    else if (!layout.leftVisible && !panel.isCollapsed()) panel.collapse()
  }, [layoutReady, layout.leftVisible, layout.leftPercent])
  return (
    <div className="flex h-svh min-h-0 flex-col overflow-hidden rounded-[var(--app-corner-radius)] bg-background text-foreground">
      <TitleBar />
      <ExpansionNotice />
      {error && (
        <Alert variant="destructive">
          <AlertTitle>{t('shell.preferencesError')}</AlertTitle>
        </Alert>
      )}
      <ResizablePanelGroup
        direction="horizontal"
        className="min-h-0 flex-1"
        onLayout={(values) => {
          if (values.length === 2) {
            sizes(values[0])
            setLayoutReady(true)
          }
        }}
      >
        <ResizablePanel
          id="left"
          order={1}
          ref={leftRef}
          defaultSize={layout.leftVisible ? layout.leftPercent : 0}
          minSize={18}
          maxSize={30}
          collapsible
          collapsedSize={0}
          onCollapse={() => visible(false)}
          onExpand={() => visible(true)}
        >
          <LeftSideBar>
            <LibraryNavigation />
          </LeftSideBar>
        </ResizablePanel>
        <ResizableHandle disabled={!layout.leftVisible} />
        <ResizablePanel
          id="main"
          order={2}
          defaultSize={100 - (layout.leftVisible ? layout.leftPercent : 0)}
          minSize={50}
        >
          <main
            className="h-full min-h-0 overflow-hidden"
            aria-label={t(view === 'home' ? 'shell.home' : 'preferences.title')}
          >
            <div hidden={view !== 'home'} className="h-full">
              <LibraryView />
            </div>
            <div hidden={view !== 'preferences'} className="h-full">
              <PreferencesPage />
            </div>
          </main>
        </ResizablePanel>
      </ResizablePanelGroup>
      <CommandPalette />
      <Toaster />
      <UpdateDialog />
    </div>
  )
}
