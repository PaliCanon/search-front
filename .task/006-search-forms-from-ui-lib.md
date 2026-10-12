# 006 The search forms come from ui-lib

status: merged · rank: 2 · level: simple · type: refactor · request: ui-lab 043's follow-up (the switch, opened at its merge 2026-10-05) · process: ../../docs/workflows/process-refactor.mmd · light path · after: [001] · branch: feat/v1-front (a commit on 001's branch, lands with 001)

## Summary
Rank: difficulty 2 (two imports, one prop, two files deleted, a `cargo update` of the git dependency), impact 2 (the same markup class for class; 043's tests guard it in ui-lib) → 2.
`components/search_bar.rs` imports `palicanon_ui_lib::{SearchCard, SearchRow}` instead of `crate::ui::{search_card, search_row}`, and passes `form_id=SEARCH_FORM_ID` where it already does (the library's `form_id` is required, no default). `app/src/ui/search_card.rs` and `search_row.rs` are deleted with their lines in `ui/mod.rs`; their SSR tests live in ui-lib now (17 tests, 7e39d28). `Cargo.lock` moves `palicanon-ui-lib` to ui-lib main at or after `7e39d28` (`cargo update -p palicanon-ui-lib`). `main.css` already `@source`s ui-lib. The page's own tests (`search_bar.rs`, `page.rs`) stay unchanged and green: the markup must not change — the served `/` and `/?q=` HTML before and after is diffed in the check (only the ui-lib path in a doc link may differ).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | orchestrator, 2026-10-05, from 043's design (slice 5) |
| Build | done | f4621f2: imports from palicanon_ui_lib, files deleted, 118 tests (10 from deleted files moved to ui-lib), fmt/clippy/rustdoc all pass, markup identical (diff: nonces and hot-reload paths only) |
| Close | done | refactor, no visible result, no errors, orchestrator check |
| Orchestrator check | done | diff read (two imports, two files deleted, ui/mod.rs); served markup identical before and after on / and /?q=; 118 tests (10 moved to ui-lib), checks 0. Cargo resolves ui-lib through the local [patch]: CI will fetch ui-lib main from GitHub, which is 7 commits behind the local main — ui-lib must be pushed before 001 merges (noted in 001). Lands with 001 |
| Check | done | user, 2026-10-12: merge with 001 |
| Merge | done | with 001 on search-front main |

## Retro
One line (light path): landed with the v1 front; nothing to carry beyond 001's retro.
