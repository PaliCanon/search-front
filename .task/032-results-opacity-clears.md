# 032 Result opacity clears after search

status: building · rank: 2 · level: simple · type: fix · request: search results stayed dim after a new search completed · light path

## Summary
Rank: difficulty 1 (one Leptos results wrapper), impact 2 (every in-place search can leave results visually dim) → 2.
Bind the results wrapper's `class` and `aria-busy` attributes reactively to the `Transition` pending signal so settled results return to normal opacity.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | `cargo fmt --all --check`; `leptosfmt` corrected the reactive attributes after CI caught formatting missed by rustfmt; SSR and hydrate checks pass with the workspace patch (`../ui-lib` v0.4.12) |
| Verify | pending | CI and the dev rollout |
| Merge | pending | |
| Production | pending | |
