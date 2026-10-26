import { useTranslation } from 'react-i18next'
import { PanelLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import appIcon from '../../../assets/brand/icon-with-bg.svg'
import { useUIStore } from '@/store/ui-store'
import { useCommandContext } from '@/hooks/use-command-context'
import { runCommand } from '@/lib/commands'
export function TitleBarLeftActions({ title = 'Texpio' }: { title?: string }) {
  const { t } = useTranslation()
  const visible = useUIStore((state) => state.layout.leftVisible)
  const context = useCommandContext()
  return (
    <>
      <img
        src={appIcon}
        alt={title}
        draggable={false}
        className="pointer-events-none size-4 shrink-0"
      />
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
    </>
  )
}
