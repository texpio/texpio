import { useTranslation } from 'react-i18next'
import { PanelLeft, PanelRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useUIStore } from '@/store/ui-store'
import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'
export function TitleBarLeftActions() {
  const { t } = useTranslation()
  const visible = useUIStore((state) => state.layout.leftVisible)
  const context = useCommandContext()
  return (
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label={t(visible ? 'titlebar.hideLeftSidebar' : 'titlebar.showLeftSidebar')}
      onClick={() => {
        void runCommand('toggle-left-sidebar', context)
      }}
    >
      <PanelLeft aria-hidden="true" />
    </Button>
  )
}
export function TitleBarRightActions() {
  const { t } = useTranslation()
  const visible = useUIStore((state) => state.layout.rightVisible)
  const context = useCommandContext()
  return (
    <Button
      variant="ghost"
      size="icon-sm"
      aria-label={t(visible ? 'titlebar.hideRightSidebar' : 'titlebar.showRightSidebar')}
      onClick={() => {
        void runCommand('toggle-right-sidebar', context)
      }}
    >
      <PanelRight aria-hidden="true" />
    </Button>
  )
}
export function TitleBarTitle({ title = 'Texpio' }: { title?: string }) {
  return (
    <div
      data-tauri-drag-region
      className="pointer-events-none absolute inset-x-1/3 text-center text-sm font-medium"
    >
      {title}
    </div>
  )
}
