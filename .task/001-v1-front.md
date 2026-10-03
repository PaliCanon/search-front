# 001 v1 front, search mocked

status: planned · rank: 5 · level: standard · type: feature · request: search-front v1 · session: search-front-001-v1 · process: ../../docs/workflows/development-process.mmd (frozen v20)

## Summary
Rank (rank.md): difficulty 5 (new repo, SSR app on ui-lib, a page with
states), impact 4 (future palicanon.app, nothing live yet) → 5.
The search page of today's site (Lsh0x/palicanon web: SearchBar, FilterBar
mode chips, ResultList, Footer), rebuilt in Leptos on ui-lib: AppShell (rail
with the search entry, theme, languages, env badge), a search bar
(role=search, / to focus), the mode chips words · meaning · both, the
results list (per-language title, snippet with highlights, found-via tag,
link to the sutta as today), empty / no-result / error states, the query in
the URL (GET ?q=&mode=) rendered on the server's first paint, desktop and
phone. The Footer comes from ui-lib (task ui-lab 034). Search is MOCKED here:
a `SearchSource` trait with an in-repo mock returning fixed hits (shape of
api /v1/search's SearchResponse / HitDto); the real client (api task 002)
plugs in later. No AI, no infra.

## Log

| Step | Result | Output |
| --- | --- | --- |
| Define | done | user (2026-10-04): "on refait d'abord le front avec les modules existants + le footer (search mock), parallélise une lib client aussi" |
