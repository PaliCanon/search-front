# 034 Drop the dev environment

status: merged
rank: 3 — removal of a whole environment; mechanical, one infra destroy to read before applying
level: simple
path: light
request: the user, 2026-11-13: « vire le dev totalement »
repository: search-front

## Summary

PaliCanon prod is moving to ssh.lsh.tech (lsh-tech, stacks/palicanon/prod).
Search is the last service still on Cloud Run prod behind its Worker; that
prod stays untouched here (migrated and removed later). The dev environment
goes completely: Cloud Run `search-front-dev`, its Worker and custom domain
`search.dev.palicanon.app`, its secrets and service account, and every line of
CI, scripts, tests and docs that served it.

## Design

- `deploy.yml`: triggered by a `v*` tag only. `ENV: prod`, `HOST: search.palicanon.app`,
  concurrency group `deploy-prod`. A push to main runs `ci.yml` and deploys nothing.
  `scripts/check_workflows.py` pins it (no `branches`, tags exactly `v*`, no dev value).
- `infra/`: `envs` holds `prod` only (default), `prod.tfvars.example` deleted (the default
  is prod). Every dev resource is the `["dev"]` instance of a `for_each`; no prod address
  or argument changes, the map keeps its shape.
- `infra/scripts/origin-secret`: `prod` only; tests moved to prod.
- App: a `K_SERVICE` ending in `-dev` is no longer `development` (it is `unknown`).
- README, infra/README: no dev environment; local-run example points at the prod API.

## Lanes

None: one lead, a small mechanical change.

## Log

| Step | Result |
| --- | --- |
| Build | check_workflows red on the real deploy.yml, then green; origin-secret test red (`create dev` accepted) then green; telemetry test red then green |
| CI fix (separate commit) | `release-values.test.sh` expected the `release_document` step to end the line; bb2af44 added `--test telemetry_browser` after it. The suite was stale, ci.yml right: fixed in the test |
| Review | pending |

## Retro

Light path, one line: main had been red since bb2af44 on a stale script assertion; a merge should read gh run list on main before the next task (quality.md).
