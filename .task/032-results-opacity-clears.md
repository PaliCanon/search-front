# 032 Result opacity clears after search

status: building · rank: 2 · level: simple · type: fix · request: search results stayed dim after a new search completed · light path

## Summary
Rank: difficulty 1 (one Leptos results wrapper), impact 2 (every in-place search can leave results visually dim) → 2.
Bind the results wrapper's `class` and `aria-busy` attributes reactively to the `Transition` pending signal so settled results return to normal opacity.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | `cargo fmt --all --check`; `cargo check -p search_front --features ssr --offline`; `cargo check -p search_front --features hydrate --target wasm32-unknown-unknown --offline` (workspace patch uses ../ui-lib v0.4.12) |
| Verify | pending | CI and the dev rollout |
| Merge | pending | |
| Production | pending | |
