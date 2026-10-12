# 008 A short page fits the phone's screen

status: merged · rank: 1 · level: simple · type: fix · request: user, 2026-10-11 ("corrige le phone avant") · process: ../../docs/workflows/process-bug.mmd · light path · branch: feat/v1-front (lands with 001)

## Summary
Rank: difficulty 1 (one layout wrapper and the footer moved to the shell's slot), impact 2 (every page on a phone) → 1.
Measured 2026-10-11 on :3101 (CDP, 390 × 844): a short page is 900 px tall — 56 px over the screen, the top bar's height; scrolled to the bottom, the search row is half under the top bar. Cause: `app/src/app.rs` `Layout` wraps the routes in its own `flex min-h-dvh flex-col` column under `AppShell`'s already screen-tall column, and renders `SiteFooter` as a child instead of through `AppShell`'s `footer` slot (ui-lib 034). Fix: drop the wrapper, pass `SiteFooter` as `footer=`. Expected: a short page is exactly the screen on a phone (scrollHeight == innerHeight), the footer at the bottom; a long page scrolls; desktop unchanged.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | user, 2026-10-11, after the orchestrator's measurement (900 px on an 844 px screen) |
| Build | done | lane (Haiku): edebe77 — the wrapper column dropped, `SiteFooter` through `AppShell`'s `footer` slot; red: `the_layout_has_no_screen_tall_column_of_its_own` (the app's `min-h-dvh` wrapper present), then green; the badge test still green; checks 0. Measured on :3101 (CDP 390 × 844): the short page 844 = 844 (was 900); home 994, 3 results 1185, 20 results 5534 (long pages scroll) |
| Close | done | one error: the lane wrote this Log into palisearch's `.task/` (c7d29a4) instead of this file — reverted there by the orchestrator; a brief gives the task file's absolute path |
| Check | done | user, 2026-10-12: merge with 001 |
| Merge | done | with 001 on search-front main |

## Retro
One line (light path): landed with the v1 front; nothing to carry beyond 001's retro.
