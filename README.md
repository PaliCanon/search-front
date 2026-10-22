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

## Inside the PaliCanon app (task 021)

The PaliCanon app (Tauri, client side, offline) mounts Search under `/search`
in its own shell, beside Learn and the Map, and searches with its local
engine only, never the network. The crate has three build modes, never two
at once: `ssr` and `hydrate` (this site) and `csr` (the app, wasm32, no
server function, no endpoint, no token: `tests/csr_tree.rs` and
`BUNDLE_MODE=csr scripts/check-wasm-no-token.sh` guard it).

The public fragment is `search_front::fragment`: `SearchProvider` (the host's
[`SearchBackend`](app/src/search/backend.rs), required) and `SearchRoutes`
(one relative route the host mounts at the root or inside its own
`ParentRoute`). Its links come from the route that matched it
(`use_matched()`), its URL is its contract (`?q=` `?c=` `?mode=` `?passage=`),
and under `csr` a submit navigates in place. Its hidden dependencies (one
language signal, root-served `/lotus.png` and `/brand/*.svg`, one provider
per page, the host's ui-lib tag, external links opened outside the window)
are in `app/src/fragment.rs` and each tested. The site plugs
`SiteBackend` (today's server function); the mode dropdown is not rendered
while the backend's `meaning()` is false.

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

It depends on `palicanon-ui-lib` by git (`tag = "v0.3.0"`), with a local
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
cargo clippy -p search_front --no-default-features --features csr --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test -p search_front --features ssr
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p search_front --features ssr
cargo leptos build                  # Verify: the bundle, then the token check
scripts/check-wasm-no-token.sh      # the built .wasm/.js hold no SEARCH_API_TOKEN, no API client
```

The app's build (`csr`) has its own browser tests, in a headless Chrome
through `wasm-bindgen-test-runner` (the version of `Cargo.lock`'s
`wasm-bindgen`) and a chromedriver matching Chrome, and its own bundle
check (the same needles plus the site's endpoint `/api/search_passages`; an
empty or unknown `BUNDLE_MODE` is an error, only an unset one is the site; CI
checks both test bundles, the app checks its own bundle the same way):

```sh
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner CHROMEDRIVER=<chromedriver> \
  cargo test -p search_front --no-default-features --features csr \
  --target wasm32-unknown-unknown --test csr --test csr_mount
# the release document tests (same runner and chromedriver):
cargo test -p search_front --no-default-features --features csr --target wasm32-unknown-unknown --test release_document
# the test binary's wasm-bindgen output holds the fragment and nothing of the server:
wasm-bindgen --target web --out-dir pkg-csr <target/wasm32-unknown-unknown/debug/deps/csr-*.wasm>
BUNDLE_MODE=csr PKG_DIR=pkg-csr scripts/check-wasm-no-token.sh
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
bash infra/scripts/lifecycle.test.sh
bash scripts/release-values.test.sh
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

The footer badge is the deploy's, not the image's: it says which ref was rolled out, so the image is identical across refs (`app/tests/no_build_git.rs`: the build reads no git). `deploy.yml` writes `SEARCH_FRONT_ENV` and `SEARCH_FRONT_SHA` onto the Cloud Run revision with `gcloud run deploy --update-env-vars`, the values from `scripts/release-values.sh`. A prod tag `v0.1.8` shows `v0.1.8` alone, `main` shows `main · <sha7>`, a local run `dev`, and a page whose metas are missing shows `unknown` in the browser. Every push to `main` makes a new revision, even with the same image. The rollout gate (`scripts/rollout-check.sh URL`, which needs `ROLLOUT_EXPECT_ENV` and `ROLLOUT_EXPECT_SHA`, the second may be empty for a tag) fails with exit 8 when the served metas differ from the intended release. The deploy uses `--update-env-vars` and never `--set-env-vars`, which would wipe the rest of the env. Tofu ignores the container env, so its apply comes before the first deploy that writes it (`infra/README.md`).

## What it never holds

The API token in any file, the WASM or the HTML (it is read from the
environment at boot and stays on the server). Content, data or screens
beyond this one search page (that is `ui-lib`'s or the corpus's); AI/chat,
instant search, the dictionary popover, an in-app reader, pagination — out
of scope for v1 (requirements, session `search-front-001-v1`).
