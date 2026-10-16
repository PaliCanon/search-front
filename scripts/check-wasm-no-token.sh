#!/usr/bin/env bash
# The token never reaches the browser: fails when the built WASM bundle or
# its JS glue (target/site/pkg/*.wasm, *.js) holds `SEARCH_API_TOKEN` (the
# server's variable), `palicanon_api_client` (the server-only client crate)
# or a canary passed as $1 (a token planted for the check). It also fails
# when the in-repo mock reached the browser: its type name (`MockSource`)
# or one of its sentinel queries (`__search_front_mock_*__`) in the bundle
# means the mock is compiled into the WASM (review 005, Δ1).
#
# It also fails on anything shaped like a PaliCanon API token, `pc_dev_` or
# `pc_prod_` + 43 base62 characters: the shape is the one line of
# `scripts/token-pattern` (what the API's `scripts/token` issues), read by
# this script and by `rollout-check.sh`. Only the file and "token shape"
# are printed, never the bytes.
#
# The PaliCanon app's bundle (the `csr` build, which searches locally and
# holds no server function) is checked with BUNDLE_MODE=csr: the same
# needles plus the server function's endpoint, `/api/search_passages`, which
# the site's own bundle legitimately holds (it is how the browser searches)
# and the app's must not (task 021). Any other mode, an empty one included,
# is exit 2; only an unset BUNDLE_MODE is the site.
#
# Run after `cargo leptos build` from the repository root:
#   scripts/check-wasm-no-token.sh [canary]
#   BUNDLE_MODE=csr PKG_DIR=<wasm-bindgen output> scripts/check-wasm-no-token.sh
# Exit 0: clean. Exit 1: a hit (the file and the needle are printed, never
# the surrounding bytes). Exit 2: nothing to check (no built bundle, or no
# token-pattern).
set -uo pipefail
# Binary files: bytes, not characters (BSD grep refuses invalid UTF-8).
export LC_ALL=C

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
pattern_file="$here/token-pattern"
if [ ! -s "$pattern_file" ]; then
  echo "check-wasm-no-token: no token-pattern next to this script" >&2
  exit 2
fi
token_pattern="$(head -n 1 "$pattern_file")"

pkg="${PKG_DIR:-target/site/pkg}"
needles=(
  SEARCH_API_TOKEN
  palicanon_api_client
  MockSource
  __search_front_mock_cold__
  __search_front_mock_error__
  __search_front_mock_rate_limited__
)
# Unset is the site; set but empty is an error (a CI variable that expanded to
# nothing must not turn an app bundle check into the site's).
case "${BUNDLE_MODE-site}" in
  site) ;;
  csr) needles+=(/api/search_passages) ;;
  *)
    echo "check-wasm-no-token: unknown BUNDLE_MODE '${BUNDLE_MODE}' (site or csr)" >&2
    exit 2
    ;;
esac
if [ -n "${1:-}" ]; then
  needles+=("$1")
fi

files=()
for f in "$pkg"/*.wasm "$pkg"/*.js; do
  [ -f "$f" ] && files+=("$f")
done
if [ "${#files[@]}" -eq 0 ]; then
  echo "check-wasm-no-token: no .wasm or .js in $pkg (run cargo leptos build)" >&2
  exit 2
fi

status=0
for f in "${files[@]}"; do
  for needle in "${needles[@]}"; do
    # -a: the .wasm is binary; -F: the needle is a literal; -q: print no bytes.
    if grep -aFq -- "$needle" "$f"; then
      if [ "$needle" = "${1:-}" ]; then shown="the canary"; else shown="$needle"; fi
      echo "check-wasm-no-token: $shown found in $f" >&2
      status=1
    fi
  done
  if grep -aEq -- "$token_pattern" "$f"; then
    echo "check-wasm-no-token: token shape found in $f" >&2
    status=1
  fi
done

if [ "$status" -eq 0 ]; then
  echo "check-wasm-no-token: ${#files[@]} files, no token, no token shape, no api client"
fi
exit "$status"
