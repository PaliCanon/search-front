# CLAUDE.md — search-front

@../docs/README.md
@../docs/conventions/index.md
@../claude-memory/MEMORY.md

This repository: the search interface of palicanon.app (search.palicanon.app
once the showcase `www` exists). Leptos SSR + hydrate on `PaliCanon/ui-lib`
(never ui-lab); calls api.palicanon.app server-side with the site token.
v1 is search only — no AI (requirements: viewer workspace `search-front`,
session `search-front-001-v1`). Prod runs on the user's server (ssh.lsh.tech, Lsh0x/lsh-tech) from a GHCR image
built by a `v*` tag and pulled by digest; `infra/` holds only the API token secret.
