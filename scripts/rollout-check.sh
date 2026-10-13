#!/usr/bin/env bash
# The deployed service is right, whoever asks: run by deploy.yml on the
# tagged revision before it takes traffic, and on the public host after.
#
#   scripts/rollout-check.sh URL [HEADER_FILE]
#       the service itself (a revision URL, with the origin guard on). The
#       secret comes from HEADER_FILE (one line, `x-origin-auth: <secret>`,
#       passed to curl as `-H @file`) or from the ORIGIN_SECRET environment
#       variable (given to curl on its standard input): never on a command
#       line, never printed.
#   scripts/rollout-check.sh --via-worker URL
#       the public host: the Worker adds the header, this script sends none
#       (and refuses to be given the secret).
#   scripts/rollout-check.sh --refused URL
#       a URL that must refuse everyone (the bare *.run.app): a 403 with an
#       empty body, without the header and with a wrong one.
#
# Checks, each named in the output: `root` (/ is 200 and names the page),
# `wasm` (the .wasm the page links is served), `search` (/?q=satipatthana is
# 200 with a result list: a live call to the API), `token` (no token shape —
# scripts/token-pattern — in the bodies and headers of / and the search),
# `guard` (without the header, or with a wrong one: 403), `refused`.
# Exit 0 all good, 1 a check failed (named), 2 usage, 3 Cloudflare answered
# with a challenge (cf-mitigated): the runner is not allowed, which says
# nothing about the service. 4 (--via-worker only) the Worker itself answered
# 500 — "The origin secret is not configured.", "The origin is not
# configured." or "The public origin is not configured." (the shared suffix
# `is not configured.`, whitespace and CR trimmed; infra/worker/proxy.js):
# the service was not reached and the remedy is `infra/scripts/origin-secret push`, not a rollback. Any other 500
# is the service's, relayed, and is a failed check (1).
set -uo pipefail
export LC_ALL=C

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
token_pattern="$(head -n 1 "$here/token-pattern")"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

usage() {
  echo "usage: rollout-check.sh [--via-worker | --refused] URL [HEADER_FILE]" >&2
  exit 2
}

mode=direct
case "${1:-}" in
  --via-worker) mode=worker; shift ;;
  --refused) mode=refused; shift ;;
esac
url="${1:-}"
header_file="${2:-}"
[ -n "$url" ] || usage
url="${url%/}"

# The secret: a file or the environment, direct mode only.
secret_mode=none
if [ "$mode" = direct ]; then
  if [ -n "$header_file" ]; then
    [ -f "$header_file" ] || { echo "rollout-check: the header file is not a file" >&2; exit 2; }
    if [ "$(grep -c '' "$header_file")" != 1 ] \
      || ! grep -Eq '^[Xx]-[Oo]rigin-[Aa]uth:[[:space:]]*[^[:space:]]+[[:space:]]*$' "$header_file"; then
      echo "rollout-check: the header file must hold one line, x-origin-auth: <secret>" >&2
      exit 2
    fi
    secret_mode=file
  elif [ -n "${ORIGIN_SECRET:-}" ]; then
    case "$ORIGIN_SECRET" in
      *[!A-Za-z0-9._~+/=-]*)
        echo "rollout-check: ORIGIN_SECRET has characters outside [A-Za-z0-9._~+/=-]" >&2
        exit 2
        ;;
    esac
    secret_mode=env
  else
    echo "rollout-check: no secret: give a HEADER_FILE or set ORIGIN_SECRET" >&2
    exit 2
  fi
elif [ -n "$header_file" ] || [ -n "${ORIGIN_SECRET:-}" ]; then
  echo "rollout-check: this mode never sends the origin secret: give none" >&2
  exit 2
fi

fail() { # check message
  echo "rollout-check: FAIL $1: $2" >&2
  exit 1
}

# fetch PATH NAME [auth|anon|wrong]: body in $work/NAME.body, headers in
# $work/NAME.head; prints the status ("000" when nothing answered).
fetch() {
  local path=$1 name=$2 who=${3:-anon}
  local args=(-sS --max-time 60 -o "$work/$name.body" -D "$work/$name.head" -w '%{http_code}')
  case "$who" in
    wrong) args+=(-H "x-origin-auth: not-the-secret") ;;
    auth)
      if [ "$secret_mode" = file ]; then
        args+=(-H "@$header_file")
      elif [ "$secret_mode" = env ]; then
        printf 'header = "x-origin-auth: %s"\n' "$ORIGIN_SECRET" \
          | curl "${args[@]}" --config - "$url$path" 2>/dev/null || printf '000'
        return
      fi
      ;;
  esac
  curl "${args[@]}" "$url$path" 2>/dev/null || printf '000'
}

# challenged NAME: exit 3 when Cloudflare's challenge, not the service, answered.
challenged() {
  if grep -iq '^cf-mitigated:' "$work/$1.head" 2>/dev/null; then
    echo "rollout-check: challenge: Cloudflare challenged this runner (cf-mitigated); the service was not reached" >&2
    exit 3
  fi
}

# workers_500 NAME STATUS: exit 4 when the Worker's own 500 (not the
# service's) answered through the public host.
workers_500() {
  [ "$mode" = worker ] && [ "$2" = 500 ] || return 0
  # Every Worker 500 body ends with `is not configured.` (proxy.js:
  # misconfigured()): matched on that suffix of the first 512 bytes, CR and
  # NUL dropped, leading and trailing whitespace trimmed, so a CRLF, a
  # trailing newline or padding added on the way does not hide it.
  local body
  body="$(head -c 512 "$work/$1.body" 2>/dev/null | tr -d '\r\0')"
  body="${body#"${body%%[![:space:]]*}"}"
  body="${body%"${body##*[![:space:]]}"}"
  if [ "${body%is not configured.}" != "$body" ]; then
    echo "rollout-check: worker-500: the Worker answered 500 (no origin secret, origin or public origin: run infra/scripts/origin-secret push); the service was not reached" >&2
    exit 4
  fi
}

expect() { # check name label status want
  if [ "$4" != "$5" ]; then
    challenged "$2"
    workers_500 "$2" "$4"
    fail "$1" "$3 answered $4 ($5 wanted)"
  fi
}

who=auth
[ "$mode" = worker ] && who=anon

if [ "$mode" = refused ]; then
  for probe in anon wrong; do
    for path in / '/?q=satipatthana'; do
      status="$(fetch "$path" "r-$probe" "$probe")"
      [ "$status" = 403 ] || { challenged "r-$probe"; fail refused "$path answered $status ($probe request; 403 wanted)"; }
      [ ! -s "$work/r-$probe.body" ] || fail refused "the 403 for $path has a body"
    done
  done
  echo "rollout-check: refused ok"
  exit 0
fi

status="$(fetch / root "$who")"
expect root root / "$status" 200
grep -aFq 'PaliCanon — Search' "$work/root.body" || fail root "/ does not name the page"
echo "rollout-check: root ok"

wasm="$(grep -aEo '/pkg/[^"'"'"' <>]*\.wasm' "$work/root.body" | head -n 1)"
[ -n "$wasm" ] || fail wasm "the page links no .wasm"
status="$(fetch "$wasm" wasm "$who")"
expect wasm wasm "$wasm" "$status" 200
echo "rollout-check: wasm ok"

status="$(fetch '/?q=satipatthana' search "$who")"
expect search search '/?q=satipatthana' "$status" 200
grep -aFq '<ol' "$work/search.body" || fail search "/?q=satipatthana has no result list"
echo "rollout-check: search ok"

for name in root search; do
  for part in body head; do
    if grep -aEq -- "$token_pattern" "$work/$name.$part"; then
      fail token "a token shape in the $part of the $name response"
    fi
  done
done
echo "rollout-check: token ok"

if [ "$mode" = direct ]; then
  status="$(fetch / open anon)"
  [ "$status" = 403 ] || { challenged open; fail guard "a request without the header answered $status (403 wanted)"; }
  status="$(fetch / wrongone wrong)"
  [ "$status" = 403 ] || fail guard "a request with a wrong header answered $status (403 wanted)"
  echo "rollout-check: guard ok"
fi
