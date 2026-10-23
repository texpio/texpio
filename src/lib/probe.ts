import { commands, requireDesktop, unwrapResult } from './tauri-bindings'
import type { Result } from './bindings'

export const profiles = ['notepad', 'vsCode', 'chromeFixture'] as const
export type ReferenceProfile = (typeof profiles)[number]
export type InsertionStrategy = 'unicode' | 'clipboard'
const phases = ['stopped', 'running', 'paused', 'fault'] as const
const strategies = ['unicode', 'clipboard'] as const
export const refusalLabels = {
  unsupportedPlatform: 'The expansion probe requires Windows.',
  unsupportedProfile: 'This control or insertion strategy is not qualified.',
  unsupportedPasswordSemantics: 'This control does not expose reliable password protection.',
  passwordField: 'Expansion is blocked in password fields.',
  readOnly: 'Expansion is blocked in readonly controls.',
  disabledControl: 'Expansion is blocked in disabled controls.',
  selection: 'Place a single caret without selected text.',
  unsupportedLayout: 'Use a qualified US or English (India) keyboard layout for this probe.',
  composition: 'Text composition is active or its state cannot be verified.',
  modifier: 'Release modifier keys before expanding.',
  integrity: 'The target must run as a standard user at the same integrity level.',
  staleContext: 'The target, caret, or input changed. Try again in the test document.',
  providerTimeout: 'The accessibility provider did not respond in time.',
  contextUnavailable: 'Reliable accessibility context is unavailable.',
  boundary: 'A shortcut must follow whitespace or the start of the field.',
  suffixMismatch: 'The exact shortcut and trailing Space could not be verified.',
  injectedInput: 'Unexpected synthetic input cancelled the request.',
  queueOverflow: 'The observation queue filled; pending requests were cancelled.',
  clipboardState: 'Clipboard preservation and paste consumption are not qualified.',
  clipboardChanged: 'A newer clipboard change cancelled the experiment.',
  outcomeUncertain: 'Output could not be verified. Check the test document before restarting.',
  busy: 'The previous native operation has not finished.',
  stopped: 'The expansion probe is stopped.',
  excluded: 'This reference app is excluded for this session.',
  shortcutConflict: 'The picker shortcut is already in use.',
  pickerUnavailable: 'The captured-target picker is unavailable.',
} as const
export type RefusalReason = keyof typeof refusalLabels
export interface ProbeStatus {
  phase: (typeof phases)[number]
  profile: ReferenceProfile | null
  strategy: InsertionStrategy
  counters: { submitted: number; observed: number; refused: number; uncertain: number }
  refusalReason: RefusalReason | null
  lastActivityAgeMs: number | null
}
export interface ProbeFixture {
  id: string
  title: string
  trigger: string
  body: string
}
export interface StartOptions {
  profile: ReferenceProfile
  strategy: InsertionStrategy
}
export const unavailableMessage = 'Windows expansion probe is unavailable.'

export class ProbeCommandError extends Error {
  constructor(public readonly code: RefusalReason) {
    super(refusalLabels[code])
    this.name = 'ProbeCommandError'
  }
}

function record(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}
function member<T extends readonly string[]>(value: unknown, choices: T): value is T[number] {
  return typeof value === 'string' && choices.includes(value)
}
function reason(value: unknown): value is RefusalReason {
  return typeof value === 'string' && Object.prototype.hasOwnProperty.call(refusalLabels, value)
}
function integer(value: unknown): value is number {
  return typeof value === 'number' && Number.isSafeInteger(value) && value >= 0
}
function parseStatus(value: unknown): ProbeStatus {
  if (
    !record(value) ||
    !member(value.phase, phases) ||
    !member(value.strategy, strategies) ||
    !(value.profile === null || member(value.profile, profiles)) ||
    !(value.refusalReason === null || reason(value.refusalReason)) ||
    !(value.lastActivityAgeMs === null || integer(value.lastActivityAgeMs)) ||
    !record(value.counters)
  )
    throw new Error(unavailableMessage)
  const { submitted, observed, refused, uncertain } = value.counters
  if (
    !integer(submitted) ||
    !integer(observed) ||
    !integer(refused) ||
    !integer(uncertain) ||
    observed > submitted
  )
    throw new Error(unavailableMessage)
  return {
    phase: value.phase,
    profile: value.profile,
    strategy: value.strategy,
    counters: { submitted, observed, refused, uncertain },
    refusalReason: value.refusalReason,
    lastActivityAgeMs: value.lastActivityAgeMs,
  }
}

async function call<T>(
  action: () => Promise<Result<unknown, unknown>>,
  parse: (value: unknown) => T,
): Promise<T> {
  try {
    requireDesktop()
    const value = unwrapResult(await action())
    return parse(value)
  } catch (error: unknown) {
    if (record(error) && reason(error.code)) throw new ProbeCommandError(error.code)
    // eslint-disable-next-line preserve-caught-error -- The IPC boundary deliberately redacts unknown native error details.
    throw new Error(unavailableMessage)
  }
}

export function getProbeStatus() {
  return call(commands.probeStatus, parseStatus)
}
export function stopProbe() {
  return call(commands.probeStop, parseStatus)
}
export async function startProbe(options: StartOptions) {
  if (!member(options?.profile, profiles) || !member(options?.strategy, strategies))
    throw new Error(unavailableMessage)
  return call(
    () => commands.probeStart({ profile: options.profile, strategy: options.strategy }),
    parseStatus,
  )
}
export async function pauseProbe(paused: boolean) {
  if (typeof paused !== 'boolean') throw new Error(unavailableMessage)
  return call(() => commands.probePause(paused), parseStatus)
}
export async function excludeProbe(profile: ReferenceProfile) {
  if (!member(profile, profiles)) throw new Error(unavailableMessage)
  return call(() => commands.probeExclude(profile), parseStatus)
}
export function getProbeFixtures(): Promise<ProbeFixture[]> {
  return call(commands.probeFixtures, (value) => {
    if (!Array.isArray(value) || value.length > 7) throw new Error(unavailableMessage)
    const ids = new Set<string>()
    return value.map((item: unknown) => {
      if (
        !record(item) ||
        !member(item.id, [
          'hello',
          'address',
          'accent',
          'devanagari',
          'emoji',
          'multiline',
          'long',
        ]) ||
        ids.has(item.id) ||
        typeof item.title !== 'string' ||
        !item.title.trim() ||
        item.title.length > 120 ||
        typeof item.trigger !== 'string' ||
        !/^;[a-z]{1,31}$/.test(item.trigger) ||
        typeof item.body !== 'string' ||
        !item.body.length ||
        item.body.length > 2000 ||
        item.body.includes('\0')
      ) {
        throw new Error(unavailableMessage)
      }
      ids.add(item.id)
      return { id: item.id, title: item.title, trigger: item.trigger, body: item.body }
    })
  })
}
