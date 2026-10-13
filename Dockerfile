# syntax=docker/dockerfile:1.7
#
# search-front: the server and the site (WASM, CSS, assets) in one image.
#
# Build context: the FOLDER HOLDING `search-front/`, `ui-lib/` AND `api/`
# (CI checks the three out side by side, `.github/actions/siblings`). The
# workspace reaches the two siblings: its `[patch]` to `../ui-lib` and to
# `../api/crates/api-client`, `app/style/main.css` (`@import` of
# `../../../ui-lib/style/theme.css`) and `app/build.rs` (`#[path]` to
# `../../ui-lib/build/placeholders.rs`). So, from that folder:
#
#   docker build -f search-front/Dockerfile .
#
# NO CREDENTIAL IS NEEDED TO BUILD (task 007, ruling R1). ui-lib and the API
# client are private git dependencies, but the `[patch]` replaces them with
# the sibling checkouts COPYed below and `Cargo.lock` records them as path
# packages: it holds no git source, so Cargo fetches nothing from GitHub. The
# GitHub App token lives in the CI checkout steps only; it is never an
# argument, a secret mount or a layer here. A RUN below fails the build if
# the lock ever gains a git source (and `app/tests/image_context.rs` pins the
# same property), instead of letting the build ask for a credential.
#
# There is no .dockerignore (it would live in the parent folder, outside this
# repository); every COPY below names what it takes instead.

# The image tag (`scripts/image-hash.sh`) hashes this file and `scripts/`
# with the sources: a fix here changes the tag.

# ---------------------------------------------------------------- tools
# The build tools, in their own layers so a source change does not reinstall
# them. 1.98 as the API's image (the MSRV of the workspace is 1.91, checked in CI).
FROM rust:1.98-bookworm AS tools
RUN rustup target add wasm32-unknown-unknown
# cargo-leptos builds the server and the WASM and runs Tailwind. It downloads
# Tailwind's standalone binary and binaryen (wasm-opt) itself, at versions it
# pins, on first use.
ARG CARGO_LEPTOS=0.3.11
RUN cargo install cargo-leptos --version "$CARGO_LEPTOS" --locked
# wasm-bindgen-cli must match the `wasm-bindgen` crate of Cargo.lock to the
# patch version, or the glue fails at build time: it is read from the lock,
# never typed twice.
COPY search-front/Cargo.lock /lock/Cargo.lock
RUN WB=$(awk '/^name = "wasm-bindgen"$/ {getline; gsub(/[^0-9.]/, ""); print; exit}' /lock/Cargo.lock) \
    && test -n "$WB" \
    && echo "wasm-bindgen-cli $WB (from Cargo.lock)" \
    && cargo install wasm-bindgen-cli --version "$WB" --locked

# ---------------------------------------------------------------- builder
FROM tools AS builder
WORKDIR /src
# ui-lib: what the workspace reads (manifest, build script, locales, theme,
# sources; README.md because a source `include_str!`s it). No .git.
COPY ui-lib/Cargo.toml ui-lib/build.rs ui-lib/README.md ./ui-lib/
COPY ui-lib/build ./ui-lib/build
COPY ui-lib/locales ./ui-lib/locales
COPY ui-lib/src ./ui-lib/src
COPY ui-lib/style ./ui-lib/style
# api: the client's own workspace (its root manifest) and the crates the
# client depends on. `api/api/` (the service) is a member of that workspace
# but nothing here reads it: `cargo metadata --locked --offline` loads the
# context without it (app/tests/image_context.rs replays these COPYs).
COPY api/Cargo.toml ./api/Cargo.toml
COPY api/crates ./api/crates
# search-front: the manifest, the lock and the one member.
COPY search-front/Cargo.toml search-front/Cargo.lock ./search-front/
COPY search-front/app ./search-front/app
COPY search-front/scripts ./search-front/scripts
WORKDIR /src/search-front
# The footer badge shows the branch and the short commit this image was built
# from, read at COMPILE time by `app/build.rs` (the image has no .git, so git
# says "unknown"). CI passes both with `--build-arg`. Declared here, after the
# COPYs, so a new commit does not invalidate the layers above.
ARG BUILD_BRANCH
ARG BUILD_SHA
ENV BUILD_BRANCH=$BUILD_BRANCH \
    BUILD_SHA=$BUILD_SHA
# The build needs no credential only while the lock holds no git source.
RUN ! grep -q '^source = "git+' Cargo.lock \
    || { echo "Cargo.lock holds a git source: the build would need a GitHub credential (task 007)"; exit 1; }
RUN cargo leptos build --release --cargo-locked \
    && scripts/check-wasm-no-token.sh \
    && mkdir /out \
    && cp target/release/search_front /out/ \
    && cp -r target/site /out/site \
    && cp scripts/smoke.sh scripts/token-pattern /out/

# ---------------------------------------------------------------- runtime
FROM debian:bookworm-slim
RUN useradd --system --uid 10001 --home /nonexistent --shell /usr/sbin/nologin app
WORKDIR /app
# smoke.sh reads token-pattern from its own folder: they travel together.
COPY --from=builder --chown=app:app /out/search_front /out/smoke.sh /out/token-pattern ./
# site/ is LEPTOS_SITE_ROOT. The bundle is not hash-named (the workspace sets
# no `hash-files`): there is no hash.txt to ship.
COPY --from=builder --chown=app:app /out/site ./site
# `get_configuration(None)` reads these and nothing else: no Cargo.toml in
# the image. Cloud Run sends its traffic to 8080. The runtime secrets
# (SEARCH_API_TOKEN, ORIGIN_SECRET) come from Secret Manager on Cloud Run
# and are not in the image. LEPTOS_ENV=PROD makes the service refuse to
# start without ORIGIN_SECRET.
ENV LEPTOS_OUTPUT_NAME=search-front \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    LEPTOS_ENV=PROD \
    RUST_LOG=info \
    PORT=8080
USER app
EXPOSE 8080
# Smoke test, the last step: `scripts/smoke.sh` starts the server as the
# image will run it, on a free local port, with a throwaway origin secret it
# makes itself (valid for this run only, never the real one) and a source
# pointing nowhere, and asks it what the rollout will ask: `/` is 200 and
# names the page, the .wasm it links is served, a request without the header
# or with a wrong one is 403, no token shape came back. A
# broken build or a missing site root fails here, not on Cloud Run. Plain
# bash (/dev/tcp): the image has no curl and gains none.
RUN ./smoke.sh ./search_front
CMD ["./search_front"]
