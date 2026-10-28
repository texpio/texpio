import { useTranslation } from 'react-i18next'
import { PanelLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useUIStore } from '@/store/ui-store'
import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'

/** Sidebar visibility toggle for page headers and the title bar. */
export function SidebarToggle() {
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
