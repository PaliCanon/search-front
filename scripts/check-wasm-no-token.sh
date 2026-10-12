#!/usr/bin/env bash
# The token never reaches the browser: fails when the built WASM bundle or
# its JS glue (target/site/pkg/*.wasm, *.js) holds `SEARCH_API_TOKEN` (the
# server's variable), `palicanon_api_client` (the server-only client crate)
# or a canary passed as $1 (a token planted for the check). It also fails
# when the in-repo mock reached the browser: its type name (`MockSource`)
# or one of its sentinel queries (`__search_front_mock_*__`) in the bundle
# means the mock is compiled into the WASM (review 005, Δ1).
#
# Run after `cargo leptos build` from the repository root:
#   scripts/check-wasm-no-token.sh [canary]
# Exit 0: clean. Exit 1: a hit (the file and the needle are printed, never
# the surrounding bytes). Exit 2: nothing to check (no built bundle).
set -uo pipefail

pkg="${PKG_DIR:-target/site/pkg}"
needles=(
  SEARCH_API_TOKEN
  palicanon_api_client
  MockSource
  __search_front_mock_cold__
  __search_front_mock_error__
  __search_front_mock_rate_limited__
)
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
done

if [ "$status" -eq 0 ]; then
  echo "check-wasm-no-token: ${#files[@]} files, no token, no api client"
fi
exit "$status"
