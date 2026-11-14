# 037 Legal and info pages move to palicanon.app

status: build
rank: 4 — difficulty 3 (two repositories, three languages of copy, a redirect and a health endpoint), impact 4 (a user-visible address change on the public site; prod hydration bug)
level: simple · light path
request: the user, 2026-11-14: « les pages légales/info appartiennent à palicanon.app, pas à search »; https://search.palicanon.app/privacy panics at hydration
repository: search-front (www: pointer file `.task/017-legal-pages-to-www.md`)

## Summary

- **www** is canonical for `/privacy`, `/terms`, `/acknowledgements` and `/support` in en/fr/vi (`/fr/…/`, `/vi/…/`), with the text that lived in search-front (`app/src/pages/content_*.rs`), carried verbatim as Markdown. `/support` merges search's text into www's existing page; `/legal` keeps its text and points to the Privacy page. The 301 rules from www to search are dropped, their tests rewritten, and the footer links the pages.
- **search-front**: `/acknowledgements`, `/terms`, `/privacy`, `/support` answer **301** to `https://palicanon.app/<lang>/<page>/` (language from the `pc_lang` cookie, else `Accept-Language`, else en; slash form so www's 307 is not added), the footer links point there, and the page code, texts, assets and locale keys are removed.
- **Root cause** of the hydration panic on `/privacy`: see "Root cause" below.
- **`GET /healthz`** → 200 `ok`, `Cache-Control: no-store`, served by the SSR server without rendering the app.

## Lanes

| lane | scope | model | status |
|---|---|---|---|
| DIAG | root cause of the panic, route-check script | sonnet | done |
| SEARCH | healthz, 301, footer, removal | sonnet | done |
| WWW-CODE | footer, redirects, tests | sonnet | done |
| CONTENT-en / fr / vi | Markdown pages, support merge | sonnet | done |
| VERIFY | every remaining route in headless Chrome, curl matrix | sonnet | running |

## Root cause

**Cloudflare's Email Address Obfuscation (Scrape Shield), not the application's code.** Cloudflare rewrites every email-shaped string in the server HTML into `<a href="/cdn-cgi/l/email-protection" class="__cf_email__" data-cfemail="…">[email&#160;protected]</a>`: one element where the hydrator expects a text or marker node. tachys 0.2.19 `hydration.rs:195` (`failed_to_cast_marker_node`, an `unreachable!()`, hence the bare "RuntimeError: unreachable" in a release wasm; a debug build says "expected a marker node, but found … a.__cf_email__").
- Hit: the pages whose text held `riviere.thibault@icloud.com`: /privacy (content_en.rs:121, :173) and /support (:299), in en, fr and vi (21 prod loads: /privacy and /support panic, `/`, `/?q=anattā`, `/terms`, `/acknowledgements`, the 404 page hydrate). Prod log line: `runtime.panic hydration.rs:195 v0.1.13`.
- Reproduced locally only behind a proxy emulating the rewrite (a plain local build hydrates fine); counter-proof: a proxy to prod that restores the plain email makes the deployed bundle hydrate on both pages.
- Not in a shared component (shell, footer, ui-lib, signals). Latent: any server-rendered text of email shape breaks the same way; the only remaining one is the zero-hit state echoing the query as text (`/?q=foo@bar.com`, result_list.rs, see the verify line in the Log).
- Fix options kept open for the user: switch Email Address Obfuscation off for the zone in Cloudflare (dashboard), which also removes the risk for any future page. www now serves these pages statically (nothing to hydrate), where the rewrite only decodes the address by script.

## Log

- 2026-11-14 Plan: lanes above, test first; no viewer session was open for this task (viewer down), so the `process` thread posts are in this log only.
