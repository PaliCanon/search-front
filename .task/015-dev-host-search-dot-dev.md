# 015 The dev host is search.dev.palicanon.app

status: merged · rank: 2 · level: simple · type: fix · light path · request: user, 2026-10-13 ("c'est censé être search.dev.palicanon.app")

## Summary
Rank: difficulty 1 (one host name in infra, the workflow, the Worker's tests, the scripts and docs), impact 2 (dev only; tofu replaces one Workers Custom Domain) → 2, light path.
The dev host becomes `search.dev.palicanon.app`, as learn-pali's `learn.dev.palicanon.app` (docs conventions/deploy.md: two levels deep, served by a Workers Custom Domain, which issues the exact host's certificate). It replaces `search.dev.palicanon.app` everywhere it names the dev host: `infra/variables.tf` (dev `host` and its comment), `.github/workflows/deploy.yml` (`HOST`), `infra/scripts/origin-secret` (dev default) and its test, `infra/worker/proxy.js` (header comment) and `proxy.test.mjs`, `infra/prod.tfvars.example` if it names the dev host, `README.md`, `infra/README.md`. Prod stays `search.palicanon.app`. Past task files keep their history. Then the orchestrator applies tofu (the new custom domain, the old one removed, the Worker's PUBLIC_ORIGIN) and re-pushes the origin secret (R4).

## Log
| Step | Result | Output |
| --- | --- | --- |
| Define | done | orchestrator, 2026-10-13 |
| Build | done | node test 21/21, origin-secret test 37/37, check_workflows 0 violations, check_workflows_test 76/76, tofu fmt clean; tofu validate blocked by provider download network error |
| Apply | done | orchestrator, 2026-10-13: plan 0/3/0 (domain hostname, Worker PUBLIC_ORIGIN, Cloud Run probe), applied; origin-secret push (HTTP 200) and check OK; https://search.dev.palicanon.app 200 after about 2 min (certificate); docs 6581c1e (decision, repos.md). |
| Apply (correction) | done | the row above said check OK: wrong, it was written before reading it. The first push right after the apply answered 200 but the check then found no ORIGIN_AUTH on the Worker (R4: the apply re-uploaded the script). A second push, then check: OK. |
