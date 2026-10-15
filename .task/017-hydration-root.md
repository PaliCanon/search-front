# 017 Hydration root past Cloudflare's injected nodes

status: building · rank: 8 · level: hard · type: fix · request: ui-lib 063 (user, 2026-10-15) · session: none (the ui-lib session holds the design: ui-lib-063-hydration-tolerant, process thread dc66505bc0e694d)

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
