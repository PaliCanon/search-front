# 002 The sutta reference in a smaller type

status: merged · rank: 1 · level: simple · type: feature · request: user, 2026-10-05 ("diminuer la police de la ref de suttas") · process: ../../docs/workflows/development-process.mmd · light path · branch: feat/v1-front (a commit on 001's branch, lands with 001)

## Summary
Rank: difficulty 1 (one class on one element), impact 1 (the look of one line of each hit) → 1.
In a hit's first line (`app/src/components/result_list.rs`, `ResultItem`), the sutta reference (`hit.display_id`, `font-mono text-xs text-gold` initially) goes down two steps from today's `text-sm`: first to `text-xs` (12px), then at the user's check to `text-[11px]` (beside the Pāli title at 17px and the title at 15px). Nothing else moves.
Model: the lane runs on Sonnet (orchestrator's override: one lane carries tasks 002, 003 and 004 in order in the same worktree, and 004 edits prose in three languages).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | user, 2026-10-05 |
| Build | done | commit f284512. Test `the_reference_is_set_one_step_smaller_than_the_titles` red first: `assertion failed: html.contains("font-mono text-xs text-gold")` (result_list.rs:254), then green after `text-sm` to `text-xs` in `ResultItem`. Checks: fmt ok, clippy ssr ok, clippy wasm32 hydrate ok, test 121 passed, doc ok. |
| Orchestrator check | done | diff read; served markup on :3101 (`?q=mindfulness`): `font-mono text-xs text-gold` on both hits; CDP captures 1280 and 390 (mobile emulation, scrollWidth = viewport) looked at. Next: the user's check |
| Check | rejected | user, 2026-10-05: "plus petite encore la ref" |
| Fix round 1 | done | ababdf7: `text-xs` to `text-[11px]`; test red then green; fmt ok, clippy ssr ok, test 128 passed, doc ok |
| Close | done | None. Voice: none (light path). |
| Check | done | user, 2026-10-05: "oui" (after the 11px fix for 002); lands with 001 |
| Check | done | user, 2026-10-12: merge with 001 |
| Merge | done | with 001 on search-front main |

## Retro
One line (light path): landed with the v1 front; nothing to carry beyond 001's retro.
