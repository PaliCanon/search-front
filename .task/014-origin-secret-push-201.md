# 014 origin-secret push: a 201 is a success; the keychain account

status: design · rank: 2 · level: simple · type: fix · light path · request: found during 007 (dev infra, 2026-10-13)

## Summary
Rank: difficulty 1 (one status check, docs), impact 2 (a false failure on every first push, a wrong command in the README) → 2.
`infra/scripts/origin-secret push dev` printed "Cloudflare answered HTTP 201; search-front-dev was not updated" although the secret was written (`check dev` → OK right after). Cloudflare answers 201 when it creates a Worker secret, 200 when it replaces one: both are success. Also `infra/README.md` and the script's comments read the token with `-a palicanon`; the keychain holds it under account `lsh` (claude-memory/secrets.md). Fix both, with a test of the status handling (the script's status branch, 200 and 201 OK, 4xx/5xx fail).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | orchestrator, 2026-10-13 |
