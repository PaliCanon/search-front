# 032 Result opacity clears after search

status: merged · rank: 2 · level: simple · type: fix · request: search results stayed dim after a new search completed · light path

## Summary
Rank: difficulty 1 (one Leptos results wrapper), impact 2 (every in-place search can leave results visually dim) → 2.
Bind the results wrapper's `class` and `aria-busy` attributes reactively to the `Transition` pending signal so settled results return to normal opacity.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | `cargo fmt --all --check`; `leptosfmt` corrected the reactive attributes after CI caught formatting missed by rustfmt; SSR and hydrate checks pass with the workspace patch (`../ui-lib` v0.4.12) |
| Verify | done | CI run 42031657318: all jobs passed, including release bundle and headless Chrome |
| Merge | done | PR #6 squash-merged as 644c1d2 on main |
| Dev | done | Workflow 42034651790: image built; revision rollout and public host/origin checks passed |
| Production | done | Tag `v0.1.9`; workflow 42041691597 passed, including the public host/origin checks |

## Retro
The busy class and `aria-busy` must read the `Transition` pending signal reactively; otherwise a completed in-place search can leave results dimmed.
