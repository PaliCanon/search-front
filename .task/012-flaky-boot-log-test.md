# 012 The boot log test fails under a shared target dir

status: design · rank: 2 · level: simple · type: fix · request: found during 010 (lane Fix 1r, 2026-10-12)

## Summary
Rank: difficulty 2 (a test harness, cause unknown), impact 1 (a red check that is not a bug, CI could hit it) → 2, light path.
`boot_process::the_real_boot_logs_its_source_once_and_no_line_holds_the_token` failed once in a combined checks run while other cargo jobs shared the target dir, then passed on two reruns (010, process thread message 5489). Find why (a spawned binary rebuilt or locked by a concurrent job? a timeout? a port?) and make it deterministic, with the reason in the test's doc.

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | found by 010's lane, 2026-10-12 |
