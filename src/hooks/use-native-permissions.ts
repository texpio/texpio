import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  getProductCapabilities,
  openPermissionSettings,
  requestProductPermissions,
  type PermissionPane,
} from '@/services/product-platform'

const key = ['product-capabilities'] as const

/**
 * One source of truth for the macOS grant state: the Settings card and the
 * launch prompt share the same query so a grant applied in System Settings
 * clears both at once.
 */
export function useNativePermissions(enabled: boolean) {
  const client = useQueryClient()
  const query = useQuery({
    queryKey: key,
    queryFn: getProductCapabilities,
    enabled,
    retry: false,
    staleTime: 5_000,
    refetchInterval: (current) =>
      current.state.data?.readiness === 'permissionRequired' ? 3_000 : false,
  })
  const request = useMutation({
    mutationFn: requestProductPermissions,
    onSuccess: (value) => client.setQueryData(key, value),
  })
  const open = useMutation({ mutationFn: openPermissionSettings })
  const capabilities = query.data
  return {
    capabilities,
    permissionsMissing: capabilities?.readiness === 'permissionRequired',
    unavailable: query.isError || request.isError,
    openFailed: open.isError,
    busy: request.isPending || open.isPending,
    checking: query.isFetching,
    request: () => request.mutate(),
    open: (pane: PermissionPane) => open.mutate(pane),
    recheck: () => void query.refetch(),
  }
}
