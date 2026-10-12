# 009 The not-found page, as today's site's

status: merged · rank: 2 · level: simple · type: feature · request: user, 2026-10-12 ("recherche dans l'ancien website, la page not found était très bien") · process: ../../docs/workflows/development-process.mmd · light path · branch: feat/v1-front (lands with 001)

## Summary
Rank: difficulty 2 (one view and four strings in three languages, the router's fallback), impact 2 (every unknown address) → 2.
Today search-front's fallback is the bare text "Not found." at the top left (status 404). The reference is today's site, `palisearch/web/src/components/Pages.tsx` `NotFoundPage` (lines 175–193) and the locales `notfound.*`, `head.notfound_title`: a centred column — the lotus and "PaliCanon" as a link to `/`, an eyebrow `404` (mono, xs, tracking-widest, subtle), the title "Anattā" (serif, 4xl, semibold, heading), the body (15px, leading-7, ink, max 42ch) "You looked for a page here and found no self in it. Like every conditioned thing, this address arose, and has already passed.", and a gold underlined link "Return to the search" to `/`. French and Vietnamese verbatim from the old locales. The head title "Anattā — PaliCanon", `noindex`. Kept: status 404, the shell (rail, footer); the column is centred in the page area (vertically within the space between the top and the footer, as the old page centred it in the screen); phone and desktop.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | user, 2026-10-12 |
| Build | done | TDD: `unknown_path_renders_not_found_page` (red → green); all tests pass; `cargo fmt`, `leptosfmt`, `clippy ssr all-targets`, `clippy hydrate wasm32 --lib`, `cargo test -p search_front --features ssr` (10 app tests, 248 total), `rustdoc -D warnings` ✓; locale keys added to parity test SAME_AS_ENGLISH (notfound_eyebrow, notfound_head_title, notfound_title) |
| Close | error | the locale files were rewritten with typographic apostrophes, five unrelated strings changed, two tests red reported as pre-existing — fixed in 5a9bc0b |
| Check | done | user, 2026-10-12: merge with 001 |
| Merge | done | with 001 on search-front main |

## Retro
One line (light path): landed with the v1 front; nothing to carry beyond 001's retro.
