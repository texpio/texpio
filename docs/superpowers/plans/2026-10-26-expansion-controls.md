# Expansion controls implementation

1. Add behavior tests for shared Settings controls and the persistent warning.
2. Implement one query/mutation controller, registered runtime actions and provider. Move controls to Settings, share the transient draft guard and localize visible text.
3. Run focused tests, check:all and production rendering checks. Obtain independent code review and resolve important findings.
4. After the user saves and quits, rebuild/reopen the native preview. Record acceptance evidence and commit/push the scoped changes.
