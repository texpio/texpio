# Writing ast-grep rules

Use ESLint for ordinary rules and ast-grep for structural boundaries. `.ast-grep/rules/` and `.ast-grep/test/` are registered by `sgconfig.yml`. Run `ast:lint` and `ast:test`.

Rules declare ID/language/severity/message, optional globs, structural patterns and metavariable constraints. Match syntax, not source text. TypeScript/TSX use separate parsers; language-specific IDs and fixtures should include generics/JSX.

Add failing invalid fixtures plus valid counterexamples first. Cover function/arrow exports, subscriptions and selector alternatives. Keep directory-scoped rules scoped. Automatic fixes require clear behavior preservation. See [official tests](https://ast-grep.github.io/guide/test-rule.html) for snapshots and validity-only mode.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
