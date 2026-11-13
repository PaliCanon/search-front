# 036 Images on GHCR, Cloud Run out of the infra

status: merged
rank: 3 — difficulty 2 (workflow rewrite and its shape rules), impact 3 (the way an image is published changes; Cloud Run is deleted by hand first)
level: simple · light path
request: the user, 2026-11-13: « fini le search, quand tout est bon on vire cloudrun » and « Passer à GHCR … Aller! »
repository: search-front

## Summary

Prod runs on the user's server (ssh.lsh.tech, Lsh0x/lsh-tech, stack
palicanon/prod), pulling the image by digest. So:

- `deploy.yml` (a `v*` tag) builds and pushes `ghcr.io/palicanon/search-front`
  (`h-<content hash>` and the release tag, the source label, a build skipped
  when the `h-` tag exists) and writes the digest to the run's summary. Every
  Cloud Run, GCP, rollout and Worker step is gone.
- `infra/` keeps only the secret `search-front-api-token-prod` (the server
  reads it). The Cloud Run service, its service account, the origin secret,
  the deployer grants, the public invoker binding and `cloudflare.tf` (with
  the Worker code and the cloudflare provider) are removed.
- Dead with them: `scripts/serving-revision.sh`, `scripts/release-values.sh`,
  `infra/scripts/*`, `scripts/infra_probe_test.py` and their CI steps; the
  workflow shape rules are rewritten for the GHCR job.
- Kept, now dead or by-hand only: the `K_SERVICE` / origin guard branches of
  the application (task 033), `scripts/rollout-check.sh` (its `--via-worker`
  mode), `scripts/smoke.sh` (the image build runs it).

## Log

- 2026-11-13: rules rewritten first (red against the old workflow), then the
  workflow and the infra; checks listed in the commit hand-back.

## Retro

Light path, one line: done from the shared GHCR brief; the first tag is the real test of the new image job.
