# 018 rollout-check waits for a cold engine

status: merged · rank: 2 · level: simple · type: bug · light path · request: found 2026-10-15 (deploy run of 017 red at the search check)

## Summary
Rank: difficulty 1 (a bounded retry in a shell check and its test), impact 3 (every dev rollout after idle time fails before traffic moves) → 2.
The pre-traffic `search` check of `scripts/rollout-check.sh` requests `/?q=satipatthana` once; the dev engine scales to zero, so after idle time the page answers "the engine is starting" with no result list, and the rollout fails (run of 2026-10-15, "FAIL search: /?q=satipatthana has no result list"). The check must retry while the page says the engine is starting: up to 12 tries, 10 s apart (lesson 2026-10-12 "the check met a cold engine"), and fail only on a page with neither results nor the starting state. A test with a fake curl that answers "starting" twice then results; one that answers "starting" every time → fail after the bound.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Found | done | orchestrator, 2026-10-15; the deploy re-run by hand after warming the engine |
| Build | done | lane (Haiku) 2be1192: retry loop on the ColdState marker `role="alert"`, ROLLOUT_SEARCH_TRIES 12 / ROLLOUT_SEARCH_WAIT 10; 3 tests red then green (27 pass); fmt, clippy 0. The lane logged no row: written by the orchestrator. |
| Orchestrator check | done | the error state shares `role="alert"`, so an error page is retried 12 times then fails with "did not start": acceptable, the failure still fires. No user check (a script). |
