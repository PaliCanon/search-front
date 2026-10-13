#!/usr/bin/env bash
# The service answers, and only to the right caller: a smoke test of the
# search-front binary, run as the last step of the image build (there is no
# curl in the image: plain bash and /dev/tcp) and by the tests.
#
#   scripts/smoke.sh BINARY
#       starts BINARY on a free local port with a throwaway ORIGIN_SECRET of
#       its own (and a real source pointing nowhere: `/` asks no one), checks
#       it, stops it.
#   ORIGIN_SECRET=… scripts/smoke.sh --port PORT
#       checks a server already listening on 127.0.0.1:PORT, with the secret
#       from the environment (never an argument).
#
# Checks, each named in the output: `listen` (the binary stays up and
# listens), `root` (/ is 200 and names the page), `guard` (a request without
# the header, or with a wrong one, is 403), `wasm` (the .wasm the page links
# is served as application/wasm), `token` (no token shape in what came
# back; the shape is scripts/token-pattern). Exit 0 all good, 1 a check
# failed (named), 2 usage (or no usable scripts/token-pattern beside it). Only status codes are printed, never a secret.
# SMOKE_WAIT: seconds to wait for the binary to listen (default 30).
set -uo pipefail
export LC_ALL=C

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# The pattern is read from the file next to this script, and an empty one
# would make `grep -E ''` match every response (task 007 F1: the image's
# runtime stage lacked the file, and a healthy page was reported as holding a
# token). A missing, empty or blank pattern is a usage error, before anything
# starts.
token_pattern=""
if [ -r "$here/token-pattern" ]; then
  token_pattern="$(head -n 1 "$here/token-pattern")"
fi
if [ -z "${token_pattern//[[:space:]]/}" ]; then
  echo "smoke: no token-pattern next to this script ($here/token-pattern is missing or empty)" >&2
  exit 2
fi
work="$(mktemp -d)"
pid=""
cleanup() {
  if [ -n "$pid" ]; then kill "$pid" 2>/dev/null; wait "$pid" 2>/dev/null; fi
  rm -rf "$work"
}
trap cleanup EXIT

fail() { # check message
  echo "smoke: FAIL $1: $2" >&2
  exit 1
}

# get PORT PATH OUT [SECRET]: the raw HTTP/1.0 response in OUT.
get() {
  (
    exec 3<>"/dev/tcp/127.0.0.1/$1" || exit 1
    {
      printf 'GET %s HTTP/1.0\r\nHost: localhost\r\n' "$2"
      if [ -n "${4:-}" ]; then printf 'x-origin-auth: %s\r\n' "$4"; fi
      printf '\r\n'
    } >&3
    cat <&3 >"$3"
  ) 2>/dev/null
}

status_of() { head -n 1 "$1" | cut -d' ' -f2; }
headers_of() { tr -d '\r' <"$1" | sed -n '1,/^$/p'; }

# check_listening PORT: waits until connected, or the binary is gone.
wait_listening() {
  local deadline=$((SECONDS + ${SMOKE_WAIT:-30}))
  until (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null; do
    if [ -n "$pid" ] && ! kill -0 "$pid" 2>/dev/null; then
      fail listen "the binary exited before it listened"
    fi
    if [ "$SECONDS" -ge "$deadline" ]; then
      fail listen "nothing listens on port $1 after ${SMOKE_WAIT:-30}s"
    fi
    sleep 0.1
  done
}

run_checks() { # port secret
  local port=$1 secret=$2 status
  get "$port" / "$work/root" "$secret" || fail root "no answer on port $port"
  status="$(status_of "$work/root")"
  [ "$status" = 200 ] || fail root "/ answered $status (200 wanted)"
  grep -aFq 'PaliCanon — Search' "$work/root" || fail root "/ does not name the page"
  echo "smoke: root ok"

  get "$port" / "$work/open" || fail guard "no answer without the header"
  status="$(status_of "$work/open")"
  [ "$status" = 403 ] || fail guard "a request without the header answered $status (403 wanted)"
  get "$port" / "$work/wrong" "not-the-secret" || fail guard "no answer with a wrong header"
  status="$(status_of "$work/wrong")"
  [ "$status" = 403 ] || fail guard "a request with a wrong header answered $status (403 wanted)"
  echo "smoke: guard ok"

  local wasm
  wasm="$(grep -aEo '/pkg/[^"'"'"' <>]*\.wasm' "$work/root" | head -n 1)"
  [ -n "$wasm" ] || fail wasm "the page links no .wasm"
  get "$port" "$wasm" "$work/wasm" "$secret" || fail wasm "no answer for $wasm"
  status="$(status_of "$work/wasm")"
  [ "$status" = 200 ] || fail wasm "$wasm answered $status (200 wanted)"
  headers_of "$work/wasm" | grep -iq '^content-type:.*wasm' \
    || fail wasm "$wasm is not served as application/wasm"
  echo "smoke: wasm ok"

  local f
  for f in root wasm; do
    if grep -aEq -- "$token_pattern" "$work/$f"; then
      fail token "a token shape in the $f response"
    fi
  done
  echo "smoke: token ok"
}

random_port() {
  local port
  # Two smoke tests started in the same second would share bash's seed.
  RANDOM=$(od -An -N2 -tu2 /dev/urandom | tr -d ' ')
  while :; do
    port=$((20000 + RANDOM % 40000))
    (exec 3<>"/dev/tcp/127.0.0.1/$port") 2>/dev/null || { echo "$port"; return; }
  done
}

case "${1:-}" in
  --port)
    [ -n "${2:-}" ] || { echo "usage: smoke.sh --port PORT (ORIGIN_SECRET in the environment)" >&2; exit 2; }
    [ -n "${ORIGIN_SECRET:-}" ] || { echo "smoke: ORIGIN_SECRET is not set" >&2; exit 2; }
    wait_listening "$2"
    run_checks "$2" "$ORIGIN_SECRET"
    ;;
  "" | -*)
    echo "usage: smoke.sh BINARY | smoke.sh --port PORT" >&2
    exit 2
    ;;
  *)
    binary="$1"
    secret="smoke-$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
    port="$(random_port)"
    ORIGIN_SECRET="$secret" SEARCH_API_URL="http://127.0.0.1:9" SEARCH_API_TOKEN="smoke-throwaway" \
      LEPTOS_SITE_ADDR="127.0.0.1:$port" "$binary" >"$work/server.log" 2>&1 &
    pid=$!
    wait_listening "$port"
    # Something else may have taken the port first: our binary must still be up.
    sleep 0.3
    kill -0 "$pid" 2>/dev/null || fail listen "the binary exited (its port was taken?)"
    run_checks "$port" "$secret"
    ;;
esac
