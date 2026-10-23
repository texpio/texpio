import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  CommandDialog,
  CommandInput,
  CommandList,
  CommandEmpty,
  CommandGroup,
  CommandItem,
  CommandShortcut,
} from '@/components/ui/command'
import { getAllCommands, runCommand } from '@/lib/commands'
import { formatShortcut } from '@/lib/menu'
import { useCommandContext } from '@/hooks/use-command-context'
import { useUIStore } from '@/store/ui-store'

function OpenPalette() {
  const { t } = useTranslation()
  const context = useCommandContext()
  const [search, setSearch] = useState('')
  const activeView = useUIStore((state) => state.activeView)
  const commands = getAllCommands(context, search).filter(
    (command) =>
      command.palette !== false && (command.group !== 'developer' || activeView === 'developer'),
  )
  const groups = [...new Set(commands.map((command) => command.group ?? 'other'))]
  return (
    <CommandDialog
      open
      onOpenChange={(open) => useUIStore.getState().setCommandPaletteOpen(open)}
      title={t('commandPalette.title')}
      description={t('commandPalette.placeholder')}
      shouldFilter={false}
    >
      <CommandInput
        value={search}
        onValueChange={setSearch}
        placeholder={t('commandPalette.placeholder')}
      />
      <CommandList>
        <CommandEmpty>{t('commandPalette.noResults')}</CommandEmpty>
        {groups.map((group) => (
          <CommandGroup key={group} heading={t(`commands.group.${group}`, { defaultValue: group })}>
            {commands
              .filter((command) => (command.group ?? 'other') === group)
              .map((command) => (
                <CommandItem
                  key={command.id}
                  value={command.id}
                  onSelect={() => {
                    useUIStore.getState().setCommandPaletteOpen(false)
                    void runCommand(command.id, context)
                  }}
                >
                  {command.icon && <command.icon aria-hidden="true" />}
                  {t(command.labelKey, { defaultValue: command.labelKey })}
                  {command.shortcut && (
                    <CommandShortcut>
                      {formatShortcut(command.shortcut, context.platform)}
                    </CommandShortcut>
                  )}
                </CommandItem>
              ))}
          </CommandGroup>
        ))}
      </CommandList>
    </CommandDialog>
  )
}
export function CommandPalette() {
  const open = useUIStore((state) => state.commandPaletteOpen)
  return open ? <OpenPalette /> : null
}
