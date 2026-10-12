# 011 Copy: no em dash in the content pages

status: design · rank: 1 · level: simple · type: fix · request: found during 010 (orchestrator check, 2026-10-12)

## Summary
Rank: difficulty 1 (strings only), impact 1 → 1, light path.
The UI copy rule (046, memory "No em dash in copy": a full stop and two sentences, never an em dash) is broken in `app/src/pages/content_{en,fr,vi}.rs`: 6, 6 and 7 em dashes on main (e.g. the app tagline "… study plans, notes — in English, French and Vietnamese."). Rewrite each with a full stop or a comma, a test that no content string holds an em dash. Waits for 010's merge (same files).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | found by the orchestrator, 2026-10-12 |
