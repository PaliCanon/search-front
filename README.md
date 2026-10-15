# search-front

The search interface of palicanon.app: one page — a search bar, the mode
chips (words · meaning · both), the results, desktop and phone — Leptos
(SSR + hydrate) on [`PaliCanon/ui-lib`](https://github.com/PaliCanon/ui-lib)
(never `ui-lab`). `?q=&mode=` in the URL, rendered on the server's first
paint: a shared link, or a plain `<form method="get">` submit, needs no
JavaScript to show the right results.

Search goes through the PaliCanon API (`api.palicanon.app`): the page's
only search path is the `search_passages` server function, which asks the
configured source on the server — the browser holds neither the mock nor
the token. Without the two variables below the service runs on the in-repo
mock (`app/src/search/mock.rs`), a few fixed hits.

## Run it

```sh
# Toolchain with the wasm32 target, isolated from the system Rust:
export RUSTUP_HOME=$HOME/.rustup-pc/rustup CARGO_HOME=$HOME/.rustup-pc/cargo PATH=$HOME/.rustup-pc/cargo/bin:$PATH RUSTC_WRAPPER=sccache
# This service's own port (task 001): site 3101, live reload 4101.
LEPTOS_SITE_ADDR=127.0.0.1:3101 LEPTOS_RELOAD_PORT=4101 \
SEARCH_API_URL=https://api-dev.palicanon.app \
SEARCH_API_TOKEN=$(security find-generic-password -a pali-canon-api -s pali-api-token-dev-site -w) \
cargo leptos watch
```

The real search needs both variables: the API's base URL and the site's
token, read from the macOS keychain in the same command (it is never
printed). Without the two (`cargo leptos watch` alone) the service runs on
the mock and says `search: mock` at boot. One of them alone, or one that
is blank, stops the service at boot with an error naming the variable
(never its value). **No `.env` file, ever**: the token lives in the
keychain, and in the process's environment for as long as it runs.

It depends on `palicanon-ui-lib` by git (`branch = "main"`), with a local
`[patch]` to `../ui-lib` (a worktree reaches it through the `wt/ui-lib`
symlink, same setup as `ui-lab`'s gallery) — a change spanning both
repositories is seen at once; a plain `cargo build` here (no patched
checkout beside it) falls back to the git branch.

## Checks

```sh
cargo fmt --all --check
leptosfmt --check .    # the view! macros (cargo install leptosfmt --locked)
cargo clippy -p search_front --features ssr --all-targets -- -D warnings
cargo clippy -p search_front --lib --features hydrate --target wasm32-unknown-unknown -- -D warnings
cargo test -p search_front --features ssr
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p search_front --features ssr
cargo leptos build                  # Verify: the bundle, then the token check
scripts/check-wasm-no-token.sh      # the built .wasm/.js hold no SEARCH_API_TOKEN, no API client
```

`scripts/check-wasm-no-token.sh [canary]` greps `target/site/pkg/*.wasm` and
`*.js` for `SEARCH_API_TOKEN`, `palicanon_api_client`, `MockSource`, the
mock's three sentinel queries (`__search_front_mock_*__`) and the canary
given as its argument (a token planted for the check); it exits 1 on a hit
and 2 when there is no bundle to check.

The same run in CI (`.github/workflows/ci.yml`), plus: `cargo check
--locked` on Rust 1.91 (the MSRV: the API client needs it), coverage, the
bundle check above, the deploy's own tests and `tofu fmt`/`validate`:

```sh
bash scripts/manifest-ref.test.sh      # the ref a manifest names for a git dependency
python3 scripts/check_workflows_test.py && python3 scripts/check_workflows.py
bash infra/scripts/origin-secret.test.sh
node --test 'infra/worker/*.test.mjs'  # the Worker
(cd infra && tofu fmt -check -recursive && tofu init -backend=false && tofu validate)
```

Functions stay under 50 lines (`clippy.toml`).

## Deploy

`main` deploys `search.dev.palicanon.app`, a `v*` tag deploys
`search.palicanon.app` (`.github/workflows/deploy.yml`, the model is
learn-pali's). Cloudflare (a Worker per host) is the only way in: the service
answers 403 to any request without the Worker's `x-origin-auth` secret, so the
`*.run.app` URL is closed. The image is built with **no credential**
(`Dockerfile`): ui-lib and the API client are COPYed as sibling checkouts, and
`Cargo.lock` holds no git source. CI's three checkouts (this repository,
`ui-lib`, `api`) use a GitHub App token (`.github/actions/siblings`); the
repository secrets are `APP_ID` and `APP_PRIVATE_KEY`, the variables
`WIF_PROVIDER`, `DEPLOYER_SA` and `GCP_PROJECT`. The infrastructure, the
Worker and the ordered first-time commands are in `infra/README.md`.
The public-host check (`scripts/rollout-check.sh --via-worker`) also loads
`/`, a search and `/privacy` in headless Chrome (`$CHROME`) and fails on a
hydration panic or a page without `<html data-hydrated>` (ui-lib 063).

## What it never holds

The API token in any file, the WASM or the HTML (it is read from the
environment at boot and stays on the server). Content, data or screens
beyond this one search page (that is `ui-lib`'s or the corpus's); AI/chat,
instant search, the dictionary popover, an in-app reader, pagination — out
of scope for v1 (requirements, session `search-front-001-v1`).
