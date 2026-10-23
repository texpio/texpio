# Bundle optimization

React Compiler runs through Babel in Vite 8's Rolldown pipeline. Both HTML entries are explicit inputs. Production builds tree-shake imports; Rust uses LTO, one codegen unit, stripping and panic abort. Existing `opt-level = 3` favors native performance; no size/speed benchmark is claimed.

Run `bun run analyze` to build and report emitted asset byte sizes. This is an asset inventory, not an interactive treemap or runtime benchmark. Compare equivalent clean builds. Use performance evidence before introducing lazy imports, splitting views or extracting shared chunks. Query devtools are development-only.

Retain reusable UI source; unused modules can be tree-shaken. Do not weaken CSP or native guards for size. Measure packaged binaries separately from web assets; platform packaging and WebView runtimes affect total delivery size.

Adapted for Texpio from [tauri-template `1295f69`](https://github.com/dannysmith/tauri-template/tree/1295f691bbdf373896e7c0f1cc969e2040fbf42c/docs), MIT. [Attribution](../../THIRD_PARTY_NOTICES.md). [Feature status](../research/tauri-template-parity.md).
