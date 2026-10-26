# Startup defaults and navigation labels

The user chose to follow the screen's aspect ratio for startup sizing. Each fresh
main-window launch starts centered at approximately 80% of the usable screen,
preserving the monitor's full-resolution aspect ratio. Grow to the existing
800×560 logical minimum where possible, but fit smaller/high-DPI work areas.
Ignore previously saved main-window geometry on startup. Manual resizing and
maximizing still work during the session; restoring a running instance does not
reset its bounds. Apply sizing immediately after the first successful show, when
Windows has finalized frameless border metrics, using a process-owned once guard.

Every normal Windows process launch, including restart and hidden sign-in launch,
attempts to start product expansion once with valid saved settings and library.
Pausing/stopping applies only to that process. Remove the conflicting startup
switch from Preferences. Retain the legacy `enabled` DTO/file field for backward
compatibility, default it to true, and ignore it for startup admission. Do not
rewrite the user's settings merely to change this policy. Corruption, native
failure and cancellation still refuse; startup must never override a newer user
Stop/Pause or start a duplicate runtime. Keep strict probe behavior separate.

Rename Unfiled to Uncategorized (including explanatory copy) and the sidebar's
Search commands to Search, translating French/Arabic equivalents. Internal folder
IDs and the command palette's behavior stay the same.

Use the existing Tauri monitor/work-area APIs and window-state skip-initial-restore
support, rather than deleting saved files or guessing screen dimensions in React.
Use native startup orchestration with fakeable unit-test boundaries; no frontend
effect arms input. Ordinary tests and headless fixtures never install hooks or
send input. The rebuilt normal preview will start expansion as explicitly requested.
