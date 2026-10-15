# 017 Hydration root past Cloudflare's injected nodes

status: merged · rank: 8 · level: hard · type: fix · request: ui-lib 063 (user, 2026-10-15) · session: none (the ui-lib session holds the design: ui-lib-063-hydration-tolerant, process thread dc66505bc0e694d)

## Summary
The site side of ui-lib 063 (`../ui-lib/.task/063-hydration-tolerates-injected-nodes.md`, design v5, nodes SITES, T3, ROLL).
Rank 8 (ui-lib 063's re-rank): a production rollout of every Leptos site, a browser-level check in the deploy.

Cause (ui-lib 063, reproduced on https://search.palicanon.app/?q=anger, 2026-10-14): Cloudflare injects a hidden `<a href="…/cdn-cgi/content?id=…">` (an AI-crawler decoy) as the FIRST child of `<body>`, and appends the Web Analytics beacon and challenge scripts after the app. `leptos::mount::hydrate_body(app::App)` (app/src/lib.rs) walks `<body>`'s children in render order, meets the decoy where it expects the app's first node, and panics (`tachys-0.2.19/src/hydration.rs:227`; release: wasm `unreachable`). The page renders but never hydrates. The user keeps both Cloudflare features (Email Obfuscation too, 2026-10-15): the code must tolerate them.

Design (ui-lib 063 v5, validated by the user 2026-10-15):
- `app/Cargo.toml`: palicanon-ui-lib `tag = "v0.1.2"` (the hydration module); `Cargo.lock` changes ui-lib's entry only.
- `app.rs shell()`: `<body><HydrationRoot><App/></HydrationRoot></body>`: the root `<div id="pc-app">` is the one element of `<body>` the server renders; the decoy and the scripts stay outside it.
- `lib.rs hydrate()`: `palicanon_ui_lib::hydrate(app::App)` instead of `hydrate_body`: hydrates from `#pc-app`, then sets `<html data-hydrated>`. The shell and the entry change in the same commit (the id written and the id read).
- T3, `scripts/rollout-check.sh --via-worker` only (Chrome cannot add the origin header), after the existing checks: check `hydrate`, headless Chrome (`$CHROME`, else `google-chrome`, `chromium`, `chromium-browser`, macOS's Google Chrome) with `--headless=new --dump-dom --virtual-time-budget=15000 --enable-logging=stderr` under a 60 s limit, on `/`, `/?q=satipatthana`, `/privacy`. Fails (exit 1, the rollback path) on `panicked at` or `RuntimeError` in Chrome's stderr, or when `data-hydrated` is missing from the dumped DOM (a timeout included). A Cloudflare challenge page in the dump exits 3, the existing challenge code: a `::warning::`, never a pass, never a rollback; the orchestrator then runs the same check by hand before any prod tag. No Chrome found: exit 2 (the runner, not the service).
- `deploy.yml`: Chrome installed on the runner (`browser-actions/setup-chrome@v2`, `CHROME` exported); the exit-3 warning says the hydrate check is the orchestrator's to run by hand.

Brief deviation, to rule: the brief asked exit 4 for a challenge in the Chrome step; 4 already means "the Worker answered 500, run origin-secret push" (deploy.yml fails the job on it), and the design (node T3SKIP) names exit 3 for a challenge. The step uses 3.

Rollout (ROLL): merge → dev deploy, T3 on search.dev.palicanon.app → green: tag v0.1.1 → prod, T3 on search.palicanon.app. T3 red on dev: no prod tag.
Out of scope: a node changed INSIDE the root (Email Obfuscation rewriting an address) still breaks hydration; T3 on /privacy guards it.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | orchestrator brief, 2026-10-15 (ui-lib 063 design v5) |
| Build | done | lane:search-front (opus), 2026-10-15, branch fix/hydration-root: 147b55f build(deps) ui-lib v0.1.2 (Cargo.lock: ui-lib 0.1.1 -> 0.1.2 only; `cargo update` also rewrote two windows-sys 0.52.0 -> 0.61.2 lines, put back, `cargo metadata --locked` exit 0), 0044b1e fix(hydration) (shell `<HydrationRoot><App/></HydrationRoot>`, entry `palicanon_ui_lib::hydrate`), e01513e feat(rollout-check) (check `hydrate`, deploy.yml setup-chrome@v2 + CHROME, README). Reds: `the_shell_body_is_the_hydration_root_around_the_app` panicked app.rs:274 "the root opens the body", exit 101; site_scripts 6 failed / 18 passed, exit 101 (the five hydrate tests and the real-binary Worker run). T3 red by hand on today's prod: `rollout-check.sh --via-worker https://search.palicanon.app` exit 1, `FAIL hydrate: /: the page panicked in the browser: … panicked at …/tachys-0.2.19/src/hydration.rs:227:9`. Checks, each alone: fmt 0, leptosfmt 0, clippy ssr all-targets 0, clippy hydrate wasm32 --lib 0, test ssr 0 (282 + 0 + 7 + 9 + 11 + 7 + 24 passed, 1 ignored), rustdoc ssr 0, `cargo leptos build --cargo-locked` 0; `check_workflows_test.py` 0 (76 tests), `check_workflows.py` 0 violations. T3 green with real Chrome on this build served locally (mock, no Cloudflare): exit 0, `hydrate ok`. Flakes seen: the first full site_scripts run after a rebuild timed out a fake Chrome at the 3 s test limit (cause not proven, a guess: the first exec of a new file on macOS under load; the limit is now 30 s, 3 s for `hang`; two full reruns green), and once the pre-existing smoke tests (`nothing listens … after 8s`, the freshly linked binary); each rerun green. Deviation to rule: a challenge in the Chrome step exits 3 (the design's T3SKIP and the script's existing challenge code), not 4 as the brief said (4 is the Worker's 500). |
| Close | done | lane:search-front, close message in the ui-lib process thread dc66505bc0e694d |
| Merge | done | ab6dd4e on main (squash), pushed: deploy.yml rolls search.dev with the hydrate check; prod by tag after the dev check. |
| Dev check | done | first deploy run red at the pre-traffic search check (cold dev engine → task 018), re-run green after warming: search-front-dev serves h-68c7fdd258adac9a; runner challenged (exit 3); orchestrator CDP check with a fresh profile: data-hydrated true, results rendered. Note: a probe reusing its Chrome profile still panicked (the old /pkg/search-front.wasm cached against the new HTML): the follow-up "hashed pkg names" (ui-lib 063, d0a3da22bb4254c) matters for returning visitors. |
| Prod | done | tag v0.1.1, run green: search-front-prod serves h-68c7fdd258adac9a; runner challenged (exit 3); orchestrator CDP check (fresh profile) on https://search.palicanon.app/?q=anger: data-hydrated true, no panic, 20 passages rendered. |

## Retro
With ui-lib 063 (retro-discussion d0f1c493b854747, docs 987f948). Follow-up: 019 hashed pkg names.
