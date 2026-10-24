import { useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { isTauri } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
import type { SnippetInfo } from '@/lib/bindings'
export const metadataKey = ['library-metadata'] as const
async function load(): Promise<SnippetInfo[]> {
  requireDesktop()
  const result = unwrapResult(await commands.libraryMetadata())
  if (
    !Array.isArray(result) ||
    result.length > 10000 ||
    result.some(
      (info) =>
        typeof info.snippetId !== 'string' ||
        typeof info.favorite !== 'boolean' ||
        !Number.isSafeInteger(info.usageCount) ||
        info.usageCount < 0 ||
        !Number.isSafeInteger(info.createdAt) ||
        !Number.isSafeInteger(info.updatedAt) ||
        (info.lastUsedAt !== null && !Number.isSafeInteger(info.lastUsedAt)),
    )
  )
    throw new Error('Metadata unavailable')
  return result
}
export function useLibraryMetadata() {
  const client = useQueryClient()
  useEffect(() => {
    const refresh = () => {
      void client.invalidateQueries({ queryKey: metadataKey })
    }
    window.addEventListener('focus', refresh)
    let cancelled = false
    let stop: (() => void) | undefined
    if (isTauri())
      void listen('library-metadata-changed', refresh).then(
        (unlisten) => {
          if (cancelled) unlisten()
          else stop = unlisten
        },
        () => {},
      )
    return () => {
      cancelled = true
      stop?.()
      window.removeEventListener('focus', refresh)
    }
  }, [client])
  return useQuery({ queryKey: metadataKey, queryFn: load, retry: false })
}
export function useSetFavorite() {
  const client = useQueryClient()
  return useMutation({
    mutationFn: async ({
      id,
      revision,
      favorite,
    }: {
      id: string
      revision: number
      favorite: boolean
    }) => {
      requireDesktop()
      unwrapResult(await commands.librarySetFavorite(id, revision, favorite))
    },
    onSuccess: () => client.invalidateQueries({ queryKey: metadataKey }),
  })
}
