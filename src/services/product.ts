import { commands, requireDesktop, unwrapResult } from '@/lib/tauri-bindings'
import type { ProductSettings, ProductStatus, ProductError } from '@/lib/bindings'

export class ProductCommandError extends Error {
  constructor(public readonly code: ProductError) {
    super('Expansion service is unavailable.')
    this.name = 'ProductCommandError'
  }
}
const errors: ProductError[] = [
  'cancelled',
  'targetChanged',
  'modifiersHeld',
  'inputBlocked',
  'clipboard',
  'busy',
  'invalidSettings',
  'unavailable',
  'attachmentMissing',
  'shortcutUnavailable',
]
export function parseProductStatus(value: unknown): ProductStatus {
  if (typeof value !== 'object' || value === null) throw new ProductCommandError('unavailable')
  const status = value as ProductStatus
  if (
    !['stopped', 'running', 'paused', 'fault'].includes(status.phase) ||
    !Number.isSafeInteger(status.submitted) ||
    status.submitted < 0 ||
    !Number.isSafeInteger(status.cancelled) ||
    status.cancelled < 0 ||
    typeof status.clipboardIncomplete !== 'boolean' ||
    !(status.lastError === null || errors.includes(status.lastError))
  )
    throw new ProductCommandError('unavailable')
  return status
}
async function call<T>(operation: () => Promise<T>): Promise<T> {
  try {
    requireDesktop()
    return await operation()
  } catch (error) {
    if (error instanceof ProductCommandError) throw error
    throw new ProductCommandError(
      errors.includes(error as ProductError) ? (error as ProductError) : 'unavailable',
    )
  }
}
export const getProductStatus = () =>
  call(async () => parseProductStatus(await commands.productStatus()))
export const startProduct = () =>
  call(async () => parseProductStatus(unwrapResult(await commands.productStart())))
export const stopProduct = () =>
  call(async () => parseProductStatus(unwrapResult(await commands.productStop())))
export const pauseProduct = () =>
  call(async () => parseProductStatus(unwrapResult(await commands.productPause())))
export const loadProductSettings = () =>
  call(async () => unwrapResult(await commands.productSettings()))
export const saveProductSettings = (settings: ProductSettings) =>
  call(async () => unwrapResult(await commands.productSaveSettings(settings)))
