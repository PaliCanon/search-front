# 027 hashed asset names

status: merged · rank: 2 · level: simple · type: fix · request: the user, 2026-10-22 («en prod le bouton settings ne marche pas ... Prio !») · light path

## Summary

After the v0.1.5 deploy a browser kept the OLD `/pkg/search-front.js` (the glue, served with `cache-control: max-age=14400` under an unhashed name) while the HTML and the wasm were new: hydration fails and every button is dead until a hard reload. Learn (learn-pali) ships hashed asset names (`hash-files = true`) and has no such problem.

Fix: the same as learn-pali. `hash-files = true` in the leptos metadata, the shell names the stylesheet, the glue and the wasm by their hashed names (`HashedStylesheet`, `HydrationScripts`, both read the hash file `cargo-leptos` writes next to the server binary), and the image ships `hash.txt` beside the binary with `LEPTOS_HASH_FILES=true`. A new build then has new file names: an old cached glue can never pair with a new wasm.

Difficulty 2, impact 4 (every visitor after a deploy); rank 2 on the light path by the orchestrator's call.

## Log

| Step | Result | Output |
| --- | --- | --- |
| Task file | done | 5ba54dd |
| Red | done | `the_document_links_the_hashed_bundle_never_the_bare_names` failed (the stylesheet linked as /pkg/search-front.css); `hashed_assets` 2 tests failed (no hash-files, no hash.txt in the image) |
| Build | done | fmt 0; leptosfmt 0; clippy-ssr 0; clippy-hydrate 0; clippy-csr 0; test 0 (330 lib + integration); doc 0; cargo leptos build --release 0; check-wasm 0; commit 4124f87 |
| Serve | done | image-like folder (binary + hash.txt + site): page links /pkg/search-front.<hash>.{css,js,wasm}, all 200; no bare name |
| Check | done | orchestrator: the curl proof (hashed wasm/js/css 200, bare links 0) and the CI matrix exit 0; the cache headers (no-cache HTML, immutable hashed files) stay in task 019 |
| Merge | done | e965005 on main, pushed (dev deploys, the image rebuilds); prod by tag v0.1.6 after the dev check |

## Retro
One line (light path): unhashed asset names plus any cache break hydration at every deploy; the user hit it first (an hour of dead buttons); task 019 had the design since 2026-10 and was never run: a known defect with a design is a task to schedule, not a note.
