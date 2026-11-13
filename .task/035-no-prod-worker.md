# 035 search.palicanon.app served by the server, not the Worker

status: merged
rank: 2 — difficulty 1 (two for_each), impact 3 (the public host moves; the server was checked first)
level: simple · light path
request: the user, 2026-11-13: « fini le search, quand tout est bon on vire cloudrun »
repository: search-front

## Summary

search-front v0.1.11 (task 033: no origin secret off Cloud Run) runs in the
lsh-tech stack (palicanon prod) and answers through its tunnel. The prod
Worker and its Workers Custom Domain leave the infra (`worker_envs` without
prod, as learn-pali and vizualisation did), so the stack can take the host's
DNS record. Cloud Run prod stays until its own removal.

## Log

- 2026-11-13: in the stack network, `/` 200, `/?q=what+is+life` 200 with
  results (the API logged two search.completed, 20 hits each); `tofu fmt`
  and `tofu validate` (scratch copy) green.

## Retro

Light path, one line: the pattern from learn-pali and vizualisation applied
as is; nothing new learned.
