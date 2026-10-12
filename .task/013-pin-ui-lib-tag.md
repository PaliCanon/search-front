# 013 Pin ui-lib by tag v0.1.1

status: merged · rank: 1 · level: simple · type: build · light path · request: user, 2026-10-12 ("publish ui-lib, et tag. Ensuite le search front")

## Summary
Rank: difficulty 1 (one manifest line and the lock), impact 2 (the build becomes reproducible) → 1, light path.
Convention (ui-lib.md, decision 2026-10-05): a service consumes ui-lib by git tag. `app/Cargo.toml` pins `branch = "main"`; it becomes `tag = "v0.1.1"` (ui-lib 0372b18, the Carousel the Support page needs). The local `[patch]` to `../ui-lib` stays.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | orchestrator, 2026-10-12 |
| Build | done | palicanon-ui-lib 0.1.0→0.1.1, windows-sys 0.61.2→0.52.0; cargo check --locked -p search_front --features ssr exit 0; cargo test --locked -p search_front --features ssr exit 0, 6 tests passed |
| Orchestrator check | done | the lane counted one test target (6); full run 257 + 3 + 6 passed, 1 ignored, exit 0; Cargo.lock: ui-lib 0.1.0 → 0.1.1 and its own windows-sys line (from ui-lib 0.1.1 manifest), nothing else. No user check: a manifest pin, no visible change. |
| Merge | done | d221ca8 on main, pushed with main. |

## Retro
The lane reported one test target's count (6) as the total; the full run had 266. Briefs already ask for exit codes; the count now asks for every `test result` line summed (task-brief, red proof's neighbour) — noted, no new rule: one occurrence.
