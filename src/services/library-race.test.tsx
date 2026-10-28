import type { PropsWithChildren } from 'react'
import { act, renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { libraryKeys, useLibrary, useSaveLibrary, type Library } from './library'

const native = vi.hoisted(() => ({ load: vi.fn(), save: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => false }))
vi.mock('@/lib/tauri-bindings', () => ({
  commands: { libraryLoad: native.load, librarySave: native.save },
  requireDesktop: () => {},
  unwrapResult: (result: { status: string; data?: unknown; error?: unknown }) => {
    if (result.status === 'ok') return result.data
    throw result.error
  },
}))

const library = (revision: number, body: string): Library => ({
  schemaVersion: 1,
  revision,
  folders: [],
  snippets: [
    {
      id: '20000000-0000-4000-8000-000000000000',
      folderId: null,
      title: 'Example',
      trigger: ';example',
      body,
      enabled: true,
      deleted: false,
    },
  ],
})
const ok = (data: Library) => ({ status: 'ok', data })
function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((done) => {
    resolve = done
  })
  return { promise, resolve }
}
function setup() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  const wrapper = ({ children }: PropsWithChildren) => (
    <QueryClientProvider client={client}>{children}</QueryClientProvider>
  )
  const hook = renderHook(() => ({ query: useLibrary(), save: useSaveLibrary() }), { wrapper })
  return { client, ...hook }
}

describe('library query and mutation ordering', () => {
  beforeEach(() => vi.resetAllMocks())

  it('does not replace a successful save with an older in-flight load', async () => {
    const original = library(4, 'original')
    const saved = library(5, 'updated')
    native.load.mockResolvedValueOnce(ok(original))
    const { client, result } = setup()
    await waitFor(() => expect(result.current.query.data).toEqual(original))

    const pendingLoad = deferred<ReturnType<typeof ok>>()
    const pendingSave = deferred<ReturnType<typeof ok>>()
    native.load.mockReturnValueOnce(pendingLoad.promise)
    native.load.mockResolvedValueOnce(ok(saved))
    native.save.mockReturnValueOnce(pendingSave.promise)
    let savePromise!: Promise<Library>
    act(() => {
      savePromise = result.current.save.mutateAsync({
        expectedRevision: 4,
        folders: [],
        snippets: saved.snippets,
      })
    })
    await waitFor(() => expect(native.save).toHaveBeenCalled())
    act(() => window.dispatchEvent(new Event('focus')))
    await waitFor(() => expect(native.load).toHaveBeenCalledTimes(2))
    await act(async () => {
      pendingSave.resolve(ok(saved))
      await savePromise
    })
    expect(client.getQueryData(libraryKeys.all)).toEqual(saved)
    await act(async () => {
      pendingLoad.resolve(ok(original))
      await pendingLoad.promise
    })
    await waitFor(() => expect(client.getQueryData(libraryKeys.all)).toEqual(saved))
    expect(client.getQueryState(libraryKeys.all)?.fetchStatus).toBe('idle')
  })

  it('allows an in-flight refresh to update the cache when the save fails', async () => {
    const original = library(4, 'original')
    const refreshed = library(5, 'external change')
    native.load.mockResolvedValueOnce(ok(original))
    const { client, result } = setup()
    await waitFor(() => expect(result.current.query.data).toEqual(original))

    const pendingLoad = deferred<ReturnType<typeof ok>>()
    native.load.mockReturnValueOnce(pendingLoad.promise)
    native.save.mockResolvedValueOnce({ status: 'error', error: { code: 'conflict' } })
    act(() => window.dispatchEvent(new Event('focus')))
    await waitFor(() => expect(native.load).toHaveBeenCalledTimes(2))
    await act(async () => {
      await expect(
        result.current.save.mutateAsync({
          expectedRevision: 4,
          folders: [],
          snippets: original.snippets,
        }),
      ).rejects.toMatchObject({ code: 'conflict' })
    })
    await act(async () => {
      pendingLoad.resolve(ok(refreshed))
      await pendingLoad.promise
    })
    await waitFor(() => expect(client.getQueryData(libraryKeys.all)).toEqual(refreshed))
  })

  it('refreshes a newer external revision after cancelling an in-flight load', async () => {
    const original = library(4, 'original')
    const saved = library(5, 'updated')
    const external = library(6, 'newer external change')
    native.load.mockResolvedValueOnce(ok(original))
    const { client, result } = setup()
    await waitFor(() => expect(result.current.query.data).toEqual(original))
    const pendingLoad = deferred<ReturnType<typeof ok>>()
    native.load.mockReturnValueOnce(pendingLoad.promise)
    native.load.mockResolvedValueOnce(ok(external))
    native.save.mockResolvedValueOnce(ok(saved))
    act(() => window.dispatchEvent(new Event('focus')))
    await waitFor(() => expect(native.load).toHaveBeenCalledTimes(2))
    await act(async () => {
      await result.current.save.mutateAsync({
        expectedRevision: 4,
        folders: [],
        snippets: saved.snippets,
      })
    })
    await act(async () => {
      pendingLoad.resolve(ok(external))
      await pendingLoad.promise
    })
    await waitFor(() => expect(client.getQueryData(libraryKeys.all)).toEqual(external))
  })

  it('does not let a delayed save response overwrite a newer library', async () => {
    const original = library(4, 'original')
    const saved = library(5, 'updated')
    const newer = library(6, 'newer external change')
    native.load.mockResolvedValueOnce(ok(original))
    const { client, result } = setup()
    await waitFor(() => expect(result.current.query.data).toEqual(original))
    const pendingSave = deferred<ReturnType<typeof ok>>()
    native.save.mockReturnValueOnce(pendingSave.promise)
    let savePromise!: Promise<Library>
    act(() => {
      savePromise = result.current.save.mutateAsync({
        expectedRevision: 4,
        folders: [],
        snippets: saved.snippets,
      })
    })
    await waitFor(() => expect(native.save).toHaveBeenCalled())
    native.load.mockResolvedValueOnce(ok(newer))
    native.load.mockResolvedValueOnce(ok(newer))
    act(() => window.dispatchEvent(new Event('focus')))
    await waitFor(() => expect(client.getQueryData(libraryKeys.all)).toEqual(newer))
    await act(async () => {
      pendingSave.resolve(ok(saved))
      await savePromise
    })
    await waitFor(() => expect(client.getQueryData(libraryKeys.all)).toEqual(newer))
  })

  it('accepts a lower revision if the library was replaced after a save', async () => {
    const original = library(4, 'original')
    const saved = library(5, 'updated')
    const recovered = library(0, 'recovered library')
    native.load.mockResolvedValueOnce(ok(original)).mockResolvedValueOnce(ok(recovered))
    native.save.mockResolvedValueOnce(ok(saved))
    const { client, result } = setup()
    await waitFor(() => expect(result.current.query.data).toEqual(original))
    await act(async () => {
      await result.current.save.mutateAsync({
        expectedRevision: 4,
        folders: [],
        snippets: saved.snippets,
      })
    })
    await waitFor(() => expect(client.getQueryData(libraryKeys.all)).toEqual(recovered))
  })
})
