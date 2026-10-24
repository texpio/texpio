type ExitGuard = () => Promise<boolean>
const guards = new Set<ExitGuard>()

/** Transient confirmations only; snippet content never enters shared layout state. */
export function registerExitGuard(guard: ExitGuard) {
  guards.add(guard)
  return () => {
    guards.delete(guard)
  }
}

export async function approveExit() {
  for (const guard of guards) {
    try {
      if (!(await guard())) return false
    } catch {
      return false
    }
  }
  return true
}
