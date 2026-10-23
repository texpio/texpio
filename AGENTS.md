# Texpio agent instructions

## Keep the user's computer available

- Prefer code, CLI, build, and test tools for development. Use Computer Use only when a native visual or interaction check is necessary.
- Keep each Computer Use session brief and focused on one small verification. Explain what will be checked before starting. Do not leave a session active while writing code, running builds/tests, waiting, or doing unrelated work.
- Use supported timeouts for UI actions. Treat an action that has not completed within about 15 seconds as stalled; do not wait indefinitely, loop on closing an app, or queue more input behind a stalled request.
- When a UI action stalls, stop sending input and end/reset the control session. Inspect the failure through read-only development tools. Reobserve the app before any later action; an unknown outcome is not permission to repeat a click or close request.
- End Computer Use promptly after the check, on failure, and before returning control to the user. Use the supported session stop/close mechanism and clear stale control bindings. Do not assume resetting JavaScript bindings alone proves that the visible Computer Use session has closed. If the available tools cannot close it, tell the user immediately rather than leaving them waiting.
- Tell the user when Computer Use has actually ended. Leave the app open only when it is needed for their preview; ending automation and closing the app are separate actions.
- If the user presses Escape or stops Computer Use, stop UI automation immediately. Do not restart it in the same turn. Resume only after the user explicitly asks to continue.
- If native UI verification cannot be completed promptly, record the exact unverified check and continue independent code/test work. Do not claim that the native check passed or enable unverified input behavior to bypass the acceptance gate.

These rules apply to every agent working in this repository, including delegated agents. Only one agent may control the desktop at a time.
