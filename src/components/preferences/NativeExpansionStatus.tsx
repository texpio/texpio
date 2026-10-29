import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { usePlatform } from '@/hooks/use-platform'
import { getProductCapabilities, requestProductPermissions } from '@/services/product-platform'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Button } from '@/components/ui/button'

const key = ['product-capabilities'] as const
export function NativeExpansionStatus() {
  const { t } = useTranslation()
  const platform = usePlatform()
  const client = useQueryClient()
  const query = useQuery({
    queryKey: key,
    queryFn: getProductCapabilities,
    enabled: platform === 'macos' || platform === 'linux',
    retry: false,
    staleTime: 5000,
  })
  const permission = useMutation({
    mutationFn: requestProductPermissions,
    onSuccess: (value) => client.setQueryData(key, value),
  })
  if (platform !== 'macos' && platform !== 'linux') return null
  const state = query.isError || permission.isError ? 'unavailable' : query.data?.readiness
  if (!state) return null
  return (
    <Alert>
      <AlertTitle>{t('product.platform.title')}</AlertTitle>
      <AlertDescription className="flex flex-col gap-2">
        {query.data && <p>{t(`product.platform.${query.data.backend}`)}</p>}
        <p>{t(`product.platform.${state}`)}</p>
        <div className="flex flex-wrap gap-2">
          {state === 'permissionRequired' && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={permission.isPending}
              onClick={() => permission.mutate()}
            >
              {t('product.platform.permissions')}
            </Button>
          )}
          <Button
            type="button"
            size="sm"
            variant="outline"
            disabled={query.isFetching}
            onClick={() => void query.refetch()}
          >
            {t('product.platform.recheck')}
          </Button>
        </div>
      </AlertDescription>
    </Alert>
  )
}
