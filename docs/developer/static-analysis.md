# Static analysis

`bun run check:all` includes typecheck, ESLint, ast-grep lint/fixtures, Prettier, Rust formatting/strict Clippy and frontend/tooling/fixture/Rust tests. `fix:all` applies formatter/lint fixes; build verification remains separate.

ESLint 10 uses compatible community React and hooks/refresh rules, not the source's incompatible old plugins. ast-grep enforces selectors and pure-library boundaries in TS/TSX with positive/negative fixtures.

Knip/jscpd are investigative reports, not deletion gates. Both entries and generated exclusions are configured. Reusable shadcn/Radix, barrels, command exports and Tauri infrastructure may be intentional. Exclude third-party/build/generated fixtures from duplication totals.

CI pins Bun/Node/Rust and uses frozen/locked installs. Native tests use fakes and never arm input. See [rule authoring](writing-ast-grep-rules.md).

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
