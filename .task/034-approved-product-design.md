# 034 — Integrate the approved PaliCanon product design

status: implemented — local validation passed
rank: 6 (coordinated across four repositories)
request: User, 2026-11-15: "Le design est validé dans l'ensemble. Adapte-le avec ce qui existe aujourd'hui."

## Scope
The original single-column result list with the approved navy, ivory and gold palette. Show full snippets inline, distinguish quoted passages from summaries, and retain filters, language, pending/error/empty states and source links.

Approved reference: `/private/tmp/palicanon-product-v2`, including the requested smaller Search type scale. Implementation uses the current applications' data and state. No new mock API, graph or progress data. English, French and Vietnamese and light/dark/system preferences remain supported.

## Coordination
Worktrees: `/private/tmp/palicanon-design-integration`; shared appearance is opt-in in ui-lib. Public app manifests must pin the exact shared-library commit before review. No release, merge or deployment requested.
Studio conversation: `g4j47w8nrm1n`; implementation thread `d327c8ebac07`, review thread `c1ea13d8ca44`.

## Validation
After restoring the original list, the native unit and integration suites passed. SSR, hydrate and CSR Clippy passed with warnings denied, followed by a fresh matched cargo-leptos server/WASM build. Real browser checks at 1728, 1024 and 390 px returned 20 results for the grief query, showed complete inline snippets and source links, and confirmed the words-only zero-results state and centered home. No horizontal overflow or browser exceptions. Evidence: `search-list-checks.json` and `classic-results-qa.json` in the integration workspace.

Shared library pinned to `204dd520d44a81499d150b2d3c4028f5f405436d`. Local-only implementation; no push, merge or deployment. Detailed logs and final evidence: `/private/tmp/palicanon-design-integration/VALIDATION.md`.

## Process exception
Automatic approval review rejected an external Claude lane because private code access for that external service had not been explicitly authorized. No external lane was launched. Implementation and verification run in the primary session. Independent persona review is not claimed.

## User follow-up
Follow-up, 2026-11-15: bounded the results and reading view to 1120 px, centered the whole workspace with equal margins, aligned the search row with the reading panel, and centered the home title, form, suggestions and daily passage. Built the stylesheet with the same Tailwind 4.2.1 binary as cargo-leptos. Browser checks at 1728, 1173 and 390 px passed with no overflow or exceptions.


## Latest user correction — original result list
The user prefers the pre-redesign results interface while retaining the new colors. Restore the original ResultList from `5e2fde1`, remove the selected-result preview and its unused translation keys, and remove the desktop split-view/clamping CSS. The search bar, filters and result list form one centered column at the original 640 px reading width (latest explicit correction: "au milieu"). The centered home and editorial shell remain in place.
