# 023 the rail lists Learn and Map

status: merged · rank: 1 · level: simple · type: fix · request: user, 2026-10-16, in the vizualisation session: «il faut rapidement corriger les redirections. que chacun des sites ai les onglets» · light path

## Summary
Rank: difficulty 1 (two entries, two keys in three catalogues, a test), impact 2 (two services missing from the rail on every page) → 1.
Every site's rail lists the three services in the same order with the same hrefs: Search (this site, `/`), Learn (https://learn.palicanon.app), Map (https://map.palicanon.app), as vizualisation's rail already does. Search's rail had only Search. Fix: the Learn and Map entries (`IconName::Grid` and `IconName::Motion` stand in, as in learn-pali and vizualisation), `nav_learn` and `nav_map` in en/fr/vi, a test of the rail (order, labels in each language, only Search current). The shared services list (one source for every rail) is a separate request, designed later. Deployed from the laptop while GitHub Actions is blocked.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | red (a mutation: Map's href changed): thread 'app::tests::the_rail_lists_search_then_learn_then_map' panicked at app/src/app.rs:221:40 (https://map.palicanon.app not found); first run alone panicked for want of an executor (the home spawns the search's resources) — the test inits it as harness::render_view does; green: fmt 0, leptosfmt 0, clippy ssr --all-targets 0, hydrate wasm32 0, csr wasm32 --all-targets 0, test ssr 0 (329 + 7 passed) |
| Merge | done | squash e387b6b (PR #3); local checks green (Actions was blocked when merged) |
| Deploy | done | tag v0.1.3; image h-749a5f3521b1a527 (Cloud Build, from the laptop) on search-front-dev and -prod: revision checks 0, traffic switched, --via-worker 0 on search.dev and search.palicanon.app (hydrate ok), direct calls refused; hydrated rail: Search → /, Learn → https://learn.palicanon.app, Map → https://map.palicanon.app |

## Retro
- **Errors**: the rails had drifted apart because each site writes its own; a first test of the rail passed in the suite only because another test had started the executor (it panicked alone) — caught by running the mutation alone.
- **Changes**: run a new test alone as well as in the suite; one source for every rail (the services list, a separate request) ends the drift.

