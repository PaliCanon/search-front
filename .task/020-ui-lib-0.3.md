# 020 search-front on ui-lib v0.3.0: the cream light theme

status: merged · rank: 2 · level: simple · type: build · request: user, 2026-10-16 ("occupe toi du search alors"; the app preview showed Search in the old palette) · light path

## Summary
Rank: difficulty 2 (the ui-lib pin v0.1.2 → v0.3.0 crosses 0.2.0, a breaking look: the cream light palette; 0.3.0 adds csr and the Store, additive), impact 2 (search.palicanon.app's light theme becomes cream like learn and palicanon.app) → 2.
search-front pins ui-lib by tag v0.1.2 (app/Cargo.toml). learn-pali made the same bump to 0.2.0 with no code change (its 043). Any compile break from 0.2/0.3 is fixed in search-front, not in ui-lib. The look is checked on the dev host after the merge, when GitHub Actions minutes are back (they are exhausted: the merge gate is the local check list, user 2026-10-16). First step of the app's Search (app-001 T8; next: 021, Search as a fragment of the app).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | 48f1fc5; cargo fmt exit 0; leptosfmt exit 0; clippy ssr exit 0; clippy hydrate exit 0; tests 27 exit 0; rustdoc exit 0; leptos build exit 0; wasm-no-token exit 0 |
| Check | done | orchestrator, 2026-10-16: diff read (the pin, README, Cargo.lock: ui-lib 0.1.2 → 0.3.0 plus windows-sys 0.52 → 0.61 in two Windows-only entries, re-resolved by `cargo update -p`; accepted); checks per the lane, all exit 0 (local list: Actions minutes exhausted, user rule). The cream is seen on search.dev after deploy, when Actions run again |
| Merge | done | fc2e637 on main, pushed (dev deploys when Actions minutes are back) |

## Retro
One line (light path): a bump with no code change; the lane needed an api worktree beside the task worktree for the `../api` patch, as the ui-lib one (the task brief named only ui-lib).
