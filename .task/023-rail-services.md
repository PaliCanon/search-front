# 023 the rail lists Learn and Map

status: open · rank: 1 · level: simple · type: fix · request: user, 2026-10-16, in the vizualisation session: «il faut rapidement corriger les redirections. que chacun des sites ai les onglets» · light path

## Summary
Rank: difficulty 1 (two entries, two keys in three catalogues, a test), impact 2 (two services missing from the rail on every page) → 1.
Every site's rail lists the three services in the same order with the same hrefs: Search (this site, `/`), Learn (https://learn.palicanon.app), Map (https://map.palicanon.app), as vizualisation's rail already does. Search's rail had only Search. Fix: the Learn and Map entries (`IconName::Grid` and `IconName::Motion` stand in, as in learn-pali and vizualisation), `nav_learn` and `nav_map` in en/fr/vi, a test of the rail (order, labels in each language, only Search current). The shared services list (one source for every rail) is a separate request, designed later. Deployed from the laptop while GitHub Actions is blocked.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | red (a mutation: Map's href changed): thread 'app::tests::the_rail_lists_search_then_learn_then_map' panicked at app/src/app.rs:221:40 (https://map.palicanon.app not found); first run alone panicked for want of an executor (the home spawns the search's resources) — the test inits it as harness::render_view does; green: fmt 0, leptosfmt 0, clippy ssr --all-targets 0, hydrate wasm32 0, csr wasm32 --all-targets 0, test ssr 0 (329 + 7 passed) |
