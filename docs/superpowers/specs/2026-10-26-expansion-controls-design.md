# Expansion controls in Settings

Move runtime controls from the library to Settings → Expansion. Rename visible Preferences text to Settings in English, French and Arabic without changing persisted navigation IDs. Healthy running expansion adds no workspace banner. Paused/stopped expansion displays a persistent, accessible top notice with Turn on expansion. Faults require explicit cleanup before starting; unavailable status offers Retry rather than guessing the engine state. Keep clipboard-restoration and native command errors visible across navigation.

One main-window controller owns polling, pending actions and errors. Both UI surfaces route through registered commands. Reuse the existing native facades/query cache; no Rust changes or new dependencies. A transient fail-closed readiness flag communicates library dirty/busy state, with no draft content or persistence. Starting requires a saved enabled non-trash snippet. Navigation must retain pending operations and errors.

Validate state transitions, duplicate-action prevention, draft/empty-library guards, fault cleanup and status recovery with synthetic native replies. Run all repository checks, production frontend build and headless light/dark/narrow/RTL checks. Replace the native preview only after a fresh save-and-Quit acknowledgement.
