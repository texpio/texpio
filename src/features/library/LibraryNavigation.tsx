import { useTranslation } from 'react-i18next'
import {
  FileText,
  Folder,
  FolderOpen,
  FolderPlus,
  Inbox,
  Pencil,
  Search,
  Settings,
  Star,
  Trash2,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { ShortcutKeys } from '@/components/ShortcutKeys'
import { useUIStore, type LibraryCollection } from '@/store/ui-store'
import { useLibrary } from '@/services/library'
import { useLibraryMetadata } from '@/services/library-metadata'
import { useCommandContext } from '@/hooks/use-command-context'
import { getAllCommands, runCommand } from '@/lib/commands'
import { resolveFolderSelection } from './model'

export function LibraryNavigation() {
  const { t } = useTranslation()
  const context = useCommandContext()
  const commands = getAllCommands(context)
  const searchShortcut = commands.find(
    (command) => command.id === 'toggle-command-palette',
  )?.shortcut
  const settingsShortcut = commands.find((command) => command.id === 'open-preferences')?.shortcut
  const accessibleShortcut = (shortcut: string | undefined) =>
    shortcut?.replace(/^CmdOrCtrl(?=\+)/i, context.platform === 'macos' ? 'Meta' : 'Control')
  const view = useUIStore((state) => state.activeView)
  const collection = useUIStore((state) => state.libraryCollection)
  const requestedFolder = useUIStore((state) => state.libraryFolder)
  const filter = useUIStore((state) => state.setLibraryFilter)
  const library = useLibrary()
  const folder = resolveFolderSelection(library.data?.folders ?? [], requestedFolder)
  const metadata = useLibraryMetadata()
  const snippets = library.data?.snippets ?? []
  const favorites = new Set(
    metadata.data?.filter((item) => item.favorite).map((item) => item.snippetId),
  )
  const collections = [
    {
      value: 'active',
      label: 'workspace.allSnippets',
      icon: FileText,
      count: snippets.filter((item) => !item.deleted).length,
    },
    {
      value: 'favorites',
      label: 'library.favorites',
      icon: Star,
      count: snippets.filter((item) => !item.deleted && favorites.has(item.id)).length,
    },
    {
      value: 'trash',
      label: 'library.trash',
      icon: Trash2,
      count: snippets.filter((item) => item.deleted).length,
    },
  ] as const
  function folderAction(action: 'new-folder' | 'manage-folder') {
    useUIStore.getState().setActiveView('home')
    void runCommand(`library-${action}`, context)
  }
  return (
    <div className="@container/navigation flex h-full min-h-0 flex-col">
      <nav
        aria-label={t('workspace.navigation')}
        className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto p-3"
      >
        <div className="flex flex-col gap-1">
          {collections.map((item) => (
            <Button
              key={item.value}
              variant={
                view === 'home' && collection === item.value && folder === 'all'
                  ? 'secondary'
                  : 'ghost'
              }
              aria-current={
                view === 'home' && collection === item.value && folder === 'all'
                  ? 'page'
                  : undefined
              }
              className="w-full justify-start"
              onClick={() => filter(item.value as LibraryCollection)}
            >
              <item.icon data-icon="inline-start" />
              <span className="min-w-0 flex-1 truncate text-start">{t(item.label)}</span>
              <span aria-hidden="true" className="text-xs tabular-nums text-muted-foreground">
                {item.count}
              </span>
            </Button>
          ))}
        </div>
        <div className="flex flex-col gap-1">
          <div className="flex items-center justify-between gap-2 px-2">
            <h2 className="text-xs font-medium text-muted-foreground">{t('workspace.folders')}</h2>
            <Button
              size="icon-sm"
              variant="ghost"
              aria-label={t('library.newFolder')}
              disabled={!library.data}
              onClick={() => folderAction('new-folder')}
            >
              <FolderPlus />
            </Button>
          </div>
          <Button
            variant={view === 'home' && folder === 'unfiled' ? 'secondary' : 'ghost'}
            className="justify-start"
            aria-current={view === 'home' && folder === 'unfiled' ? 'page' : undefined}
            onClick={() => filter('active', 'unfiled')}
          >
            <Inbox data-icon="inline-start" />
            <span className="truncate">{t('library.unfiled')}</span>
          </Button>
          {library.data?.folders.map((item) => (
            <div key={item.id} className="flex min-w-0 items-center gap-1">
              <Button
                variant={view === 'home' && folder === item.id ? 'secondary' : 'ghost'}
                className="min-w-0 flex-1 justify-start"
                aria-current={view === 'home' && folder === item.id ? 'page' : undefined}
                onClick={() => filter('active', item.id)}
              >
                {folder === item.id ? (
                  <FolderOpen data-icon="inline-start" />
                ) : (
                  <Folder data-icon="inline-start" />
                )}
                <span className="truncate">{item.name}</span>
              </Button>
              {view === 'home' && folder === item.id && (
                <Button
                  size="icon-sm"
                  variant="ghost"
                  aria-label={t('library.manageFolder')}
                  onClick={() => folderAction('manage-folder')}
                >
                  <Pencil />
                </Button>
              )}
            </div>
          ))}
        </div>
      </nav>
      <div className="flex shrink-0 flex-col gap-1 border-t p-3">
        <Button
          variant="ghost"
          className="min-w-0 justify-start"
          aria-keyshortcuts={accessibleShortcut(searchShortcut)}
          onClick={() => {
            void runCommand('toggle-command-palette', context)
          }}
        >
          <Search data-icon="inline-start" />
          <span className="min-w-0 flex-1 truncate text-start">{t('workspace.commands')}</span>
          {searchShortcut && (
            <span
              aria-hidden="true"
              className="ms-auto hidden shrink-0 @min-[12rem]/navigation:inline-flex"
            >
              <ShortcutKeys shortcut={searchShortcut} platform={context.platform} />
            </span>
          )}
        </Button>
        <Button
          variant={view === 'preferences' ? 'secondary' : 'ghost'}
          className="min-w-0 justify-start"
          aria-current={view === 'preferences' ? 'page' : undefined}
          aria-keyshortcuts={accessibleShortcut(settingsShortcut)}
          onClick={() => {
            void runCommand('open-preferences', context)
          }}
        >
          <Settings data-icon="inline-start" />
          <span className="min-w-0 flex-1 truncate text-start">{t('preferences.title')}</span>
          {settingsShortcut && (
            <span
              aria-hidden="true"
              className="ms-auto hidden shrink-0 @min-[12rem]/navigation:inline-flex"
            >
              <ShortcutKeys shortcut={settingsShortcut} platform={context.platform} />
            </span>
          )}
        </Button>
      </div>
    </div>
  )
}
