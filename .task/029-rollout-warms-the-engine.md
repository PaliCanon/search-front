# 029 The rollout check warms the search engine before it judges a result list

status: merged · rank: 2 · level: simple · type: fix · request: found by the orchestrator, 2026-10-22 (three dev deploys of the day failed their first rollout check with `rollout-check: FAIL search: /?q=satipatthana has no result list`, and passed on a re-run: the dev engine scales to zero and answers Cold at the first request) · light path

## Summary
Rank: difficulty 2 (one script), impact 3 (every deploy after a quiet hour fails once and needs a hand re-run) → 2.
`scripts/rollout-check.sh`'s search step reads the tagged URL's `/?q=satipatthana` once and fails on a Cold answer. Fix: before the judgement, warm the backend the way 005 and deploy.md say (a first request whose answer is read: a Cold state is followed by up to N retries at 10 s until a result list or the deadline, 2 min), then judge; the log says how many warm-up requests it took. Test in the script's suite with a fake page that answers Cold twice then results; a fake that never warms still exits with the search failure. CI step unchanged (the suite runs already).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Build | done | f7d1549; cargo test 36 passed exit 0; cargo fmt exit 0; bash -n exit 0 |
| Check | done | orchestrator: the retry covers a 503 and a 200 page without a result list; 36 site_scripts tests, fmt, bash -n exit 0; a deploy-time script, visible at the next deploy's log |
| Merge | done | 2739627 on main, pushed (a deploy-time script: the dev deploy skips the image; prod takes it at the next tag) |

## Retro
One line (light path): a rollout gate that reads a scale-to-zero backend warms it first (deploy.md, 005); the lane first read Cold as a 503 while the real page was a 200 without results: the brief should quote the real failure line.
