# 022 Support page: the tools to discover the Canon (search, learn, map)

status: design · rank: 5 · level: standard · type: feature · session: search-front-022-support-tools · process: ../../docs/workflows/development-process.mmd

## Summary
Rank: difficulty 4 (three showcase blocks on one page, content in three languages, screenshots and carousels reused from ui-lib's Carousel; the learn and map screenshots exist in www's design task), impact 5 (the Support page is linked from every page's footer) → 5. Orchestrator: a Sonnet lead; design on Opus.
Request (user, 2026-10-16): "Il faudrait aussi mettre à jour la page support pour présenter les différents tools pour découvrir le palicanon. … ça doit être search / learn / map." Today `app/src/pages/content_{en,fr,vi}.rs` `support` holds a search showcase (five screenshots, `app/public/support/search-*.jpg`), the Tipiṭaka app's (five, `app-*.jpg`), the Instagram block, then the sections (what it costs, Ko-fi, other ways to help); `app/src/pages/view.rs` renders the showcases with `palicanon_ui_lib::Carousel` (search first, then the app: 010).
To design: the three tools in order — the search (search.palicanon.app), learning Pāli (learn.palicanon.app), the map (map.palicanon.app) — each a showcase block (heading, one tagline, a button to the service, a carousel of screenshots); whether the app's block and the Instagram block stay (open, the user decides); the screenshots for learn and map (www 004 captured both: `www/src/assets/screens/{viz,learn}/`; learn's from learn.dev); the strings in the three languages; the `SearchBlock`/`AppBlock` structs generalised into one `ToolBlock` (a list in order) rather than a third optional field.
Open: keep the app block? keep Instagram? the order (search, learn, map)? the map's screenshots before it is deployed (the www captures come from a local run).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | doing | orchestrator, 2026-10-16: the need posted, the user asked about the app and Instagram blocks |
