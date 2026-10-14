# 016 The startup probe's timeout is shorter than its period

status: merged · rank: 2 · level: simple · type: fix · light path · request: found during 007's second deploy run (2026-10-13)

## Summary
Rank: difficulty 1 (three numbers in tofu and a test), impact 2 (every dev rollout fails until fixed) → 2, light path.
Deploy run 38419089828: the image built, passed its smoke and was pushed, then `gcloud run deploy` refused the service: `startup_probe.timeout_seconds: must be less than period_seconds`. `infra/gcp.tf:143` sets `period_seconds = 2`, `timeout_seconds = 3`, `failure_threshold = 30` (the Cloud Run v2 API took it at create; gcloud validates it at deploy). Ruled: `period_seconds = 5`, `timeout_seconds = 3`, `failure_threshold = 12` — the same 60 s margin for a slow cold pull, timeout below period. A test pins the rule (timeout < period, period × threshold ≥ 60) on the tofu file, beside the existing infra checks. Built on the same branch as 015 (one apply for both).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | orchestrator, 2026-10-13 |
| Build | done | test: period=2, timeout=3, threshold=30: timeout not < period (1 fail, 1 pass); then period=5, timeout=3, threshold=12: both pass (2/2); infra_probe_test.py, check_workflows_test.py (76/76), check_workflows.py, tofu fmt all green |
| Apply | done | with 015 (one apply): Cloud Run startup probe 5 s / 3 s / 12, in place. |

## Retro
Shared with 015 (see its Retro).
- **Went wrong**: the probe (timeout ≥ period) passed tofu plan and 007's review — the Cloud Run v2 API accepts it, `gcloud run deploy` does not — and failed the first real rollout (run 38419089828).
- **Went well**: cause named from the run's log; the 60 s cold-pull margin kept (5 s × 12); test written first, red on the old values, green after, wired into ci.yml.
- **Changes**: infra reviews check each Cloud Run field against gcloud's rules, not only the API's; a rule a rollout rejects gets a pinned test (done here).
