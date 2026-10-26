import { useState } from 'react'
import { isTauri } from '@tauri-apps/api/core'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import type { Library, ProductError, ProductStatus } from '@/lib/bindings'
import type { ExpansionAction } from '@/lib/commands/expansion-commands'
import { useUIStore } from '@/store/ui-store'
import { libraryKeys, loadLibrary } from './library'
import {
  getProductStatus,
  pauseProduct,
  ProductCommandError,
  startProduct,
  stopProduct,
} from './product'

const mutationKey = ['expansion-control'] as const
export function useExpansionController(startBlocked: boolean) {
  const client = useQueryClient()
  const desktop = isTauri()
  const [error, setError] = useState<ProductError | null>(null)
  const status = useQuery({
    queryKey: libraryKeys.engine,
    queryFn: getProductStatus,
    enabled: desktop,
    retry: false,
    refetchInterval: () => (client.isMutating({ mutationKey }) ? false : 1000),
  })
  const library = useQuery({
    queryKey: libraryKeys.all,
    queryFn: loadLibrary,
    enabled: desktop,
    retry: false,
  })
  const mutation = useMutation({
    mutationKey,
    onMutate: () => client.cancelQueries({ queryKey: libraryKeys.engine }),
    mutationFn: (action: ExpansionAction) =>
      action === 'start' ? startProduct() : action === 'pause' ? pauseProduct() : stopProduct(),
    onSuccess: (next) => {
      client.setQueryData(libraryKeys.engine, next)
      setError(null)
    },
    onError: (reason) =>
      setError(reason instanceof ProductCommandError ? reason.code : 'unavailable'),
    onSettled: () => client.invalidateQueries({ queryKey: libraryKeys.engine }),
  })
  const eligible = (data: Library | undefined) =>
    !!data?.snippets.some((snippet) => snippet.enabled && !snippet.deleted)
  const can = (action: ExpansionAction) => {
    if (!desktop || status.isError || mutation.isPending || client.isMutating({ mutationKey }))
      return false
    const phase = client.getQueryData<ProductStatus>(libraryKeys.engine)?.phase
    if (action === 'start')
      return (
        (phase === 'stopped' || phase === 'paused') &&
        !useUIStore.getState().expansionStartBlocked &&
        !library.isError &&
        eligible(client.getQueryData<Library>(libraryKeys.all))
      )
    if (action === 'pause') return phase === 'running'
    return phase === 'running' || phase === 'paused' || phase === 'fault'
  }
  const startHint:
    | 'expansion.saveFirst'
    | 'expansion.libraryUnavailable'
    | 'library.loading'
    | 'expansion.needsSnippet'
    | null = startBlocked
    ? 'expansion.saveFirst'
    : library.isError
      ? 'expansion.libraryUnavailable'
      : !library.data
        ? 'library.loading'
        : !eligible(library.data)
          ? 'expansion.needsSnippet'
          : null
  return {
    desktop,
    status,
    error,
    can,
    startHint,
    pending: mutation.isPending,
    pendingAction: mutation.isPending ? mutation.variables : null,
    act: (action: ExpansionAction) => {
      if (can(action)) mutation.mutate(action)
    },
    retry: () => {
      void status.refetch()
      void library.refetch()
    },
  }
}
