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
#       The release it must serve (task 028) comes from the environment:
#       ROLLOUT_EXPECT_ENV (non-empty: `main`, a branch name, or a tag) and
#       ROLLOUT_EXPECT_SHA (must be SET; empty for a tag, which has no sha).
#       Either missing is a usage error (2) before any request: a gate that
#       silently skips is no gate. The other modes neither read nor need them.
#   scripts/rollout-check.sh --via-worker URL
#       the public host: the Worker adds the header, this script sends none
#       (and refuses to be given the secret).
#   scripts/rollout-check.sh --refused URL
#       a URL that must refuse everyone (the bare *.run.app): a 403 with an
#       empty body, without the header and with a wrong one.
#
# Checks, each named in the output: `root` (/ is 200 and names the page),
# `release` (direct mode only: the page's `search-release-env` and
# `search-release-sha` metas equal ROLLOUT_EXPECT_ENV and ROLLOUT_EXPECT_SHA:
# the revision serves the release the deploy meant), `wasm` (the .wasm the page links is served), `search` (/?q=satipatthana is
# 200 with a result list: a live call to the API), `token` (no token shape —
# scripts/token-pattern — in the bodies and headers of / and the search),
# `guard` (without the header, or with a wrong one: 403), `refused`, and,
# through the Worker only (Chrome cannot add the origin header), `hydrate`:
# headless Chrome loads /, /?q=satipatthana and /privacy (ui-lib 063: the
# page hydrates past the nodes Cloudflare injects); a page passes when
# Chrome's console (stderr) has no `panicked at` nor `RuntimeError` and its
# dumped DOM has `<html data-hydrated>` (set by palicanon_ui_lib::hydrate).
# Chrome is $CHROME, else google-chrome, google-chrome-stable, chromium,
# chromium-browser or macOS's Google Chrome; each page gets
# $ROLLOUT_CHROME_TIMEOUT seconds (60), past which it fails.
# Exit 0 all good, 1 a check failed (named), 8 (direct mode only) the `release`
# check failed: the revision serves another release than the intended one, or
# no release metas, 2 usage (or no usable
# token-pattern beside the script, or no usable Chrome), 3 Cloudflare answered
# with a challenge (cf-mitigated, or a challenge page in Chrome): the runner
# is not allowed, which says nothing about the service, and is never a pass. 4 (--via-worker only) the Worker itself answered
# 500 — "The origin secret is not configured.", "The origin is not
# configured." or "The public origin is not configured." (the shared suffix
# `is not configured.`, whitespace and CR trimmed; infra/worker/proxy.js):
# the service was not reached and the remedy is `infra/scripts/origin-secret push`, not a rollback. Any other 500
# is the service's, relayed, and is a failed check (1).
set -uo pipefail
export LC_ALL=C

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# A missing, empty or blank pattern would make `grep -E ''` match every
# response (task 007 F5, as smoke.sh F1): a usage error, before any request.
token_pattern=""
if [ -r "$here/token-pattern" ]; then
  token_pattern="$(head -n 1 "$here/token-pattern")"
fi
if [ -z "${token_pattern//[[:space:]]/}" ]; then
  echo "rollout-check: no token-pattern next to this script ($here/token-pattern is missing or empty)" >&2
  exit 2
fi
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
    sed -E 's/^[Xx]-[Oo]rigin-[Aa]uth:[[:space:]]*//; s/[[:space:]]*$//' "$header_file" > "$work/secret.value"
  elif [ -n "${ORIGIN_SECRET:-}" ]; then
    case "$ORIGIN_SECRET" in
      *[!A-Za-z0-9._~+/=-]*)
        echo "rollout-check: ORIGIN_SECRET has characters outside [A-Za-z0-9._~+/=-]" >&2
        exit 2
        ;;
    esac
    secret_mode=env
    printf '%s\n' "$ORIGIN_SECRET" > "$work/secret.value"
  else
    echo "rollout-check: no secret: give a HEADER_FILE or set ORIGIN_SECRET" >&2
    exit 2
  fi
  if [ -z "${ROLLOUT_EXPECT_ENV:-}" ]; then
    echo "rollout-check: usage: the direct mode needs ROLLOUT_EXPECT_ENV, the intended release env (non-empty: main, a branch name or a tag)" >&2
    exit 2
  fi
  if [ -z "${ROLLOUT_EXPECT_SHA+set}" ]; then
    echo "rollout-check: usage: the direct mode needs ROLLOUT_EXPECT_SHA set (empty for a tag, which has no sha)" >&2
    exit 2
  fi
elif [ -n "$header_file" ] || [ -n "${ORIGIN_SECRET:-}" ]; then
  echo "rollout-check: this mode never sends the origin secret: give none" >&2
  exit 2
fi

die() { # exit-code check message
  echo "rollout-check: FAIL $2: $3" >&2
  exit "$1"
}
fail() { die 1 "$1" "$2"; }

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

# meta_content NAME: prints the content attribute of the first <meta> tag whose
# name attribute is NAME, whatever the order of its attributes (the tag may
# span lines). Returns 1 when there is no such tag, 2 when it has no content.
meta_content() {
  local tag found=""
  while IFS= read -r tag; do
    if grep -Eq "[[:space:]]name=(\"$1\"|'$1')" <<<"$tag"; then
      found=$tag
      break
    fi
  done < <(tr '\r\n' '  ' < "$work/root.body" | grep -aEo '<meta[[:space:]][^>]*>')
  [ -n "$found" ] || return 1
  local attr
  attr="$(grep -aEo "[[:space:]]content=(\"[^\"]*\"|'[^']*')" <<<"$found" | head -n 1)"
  [ -n "$attr" ] || return 2
  attr="${attr#*content=}"
  printf '%s' "${attr:1:${#attr}-2}"
}

# shown VALUE: a served value for a message: printable, short, and never the
# origin secret (a page that echoes it must not make this script print it).
shown() {
  if grep -aFq -f "$work/secret.value" <<<"$1"; then
    printf '<the origin secret>'
  else
    printf '%s' "$1" | tr -cd '[:print:]' | head -c 100
  fi
}

# check_release META WANT: the served meta must equal the intended value.
check_release() {
  local served rc=0
  served="$(meta_content "$1")" || rc=$?
  case "$rc" in
    1) die 8 release "the page has no <meta name=\"$1\"> (intended '$2')" ;;
    2) die 8 release "the <meta name=\"$1\"> has no content attribute (intended '$2')" ;;
  esac
  [ "$served" = "$2" ] \
    || die 8 release "$1: expected '$2', the page serves '$(shown "$served")'"
}

if [ "$mode" = direct ]; then
  check_release search-release-env "$ROLLOUT_EXPECT_ENV"
  check_release search-release-sha "$ROLLOUT_EXPECT_SHA"
  echo "rollout-check: release ok"
fi

wasm="$(grep -aEo '/pkg/[^"'"'"' <>]*\.wasm' "$work/root.body" | head -n 1)"
[ -n "$wasm" ] || fail wasm "the page links no .wasm"
status="$(fetch "$wasm" wasm "$who")"
expect wasm wasm "$wasm" "$status" 200
echo "rollout-check: wasm ok"

# Search check with retry for a cold engine.
# The engine may be starting (page has role="alert"), and we should retry.
ROLLOUT_SEARCH_TRIES="${ROLLOUT_SEARCH_TRIES:-12}"
ROLLOUT_SEARCH_WAIT="${ROLLOUT_SEARCH_WAIT:-10}"

for try in $(seq 1 "$ROLLOUT_SEARCH_TRIES"); do
  status="$(fetch '/?q=satipatthana' search "$who")"
  expect search search '/?q=satipatthana' "$status" 200

  # Check if we have a result list
  if grep -aFq '<ol' "$work/search.body"; then
    echo "rollout-check: search ok"
    break
  fi

  # Check if the page shows the engine is starting
  if grep -aFq 'role="alert"' "$work/search.body"; then
    if [ "$try" -lt "$ROLLOUT_SEARCH_TRIES" ]; then
      echo "rollout-check: search: the engine is starting, try $try/$ROLLOUT_SEARCH_TRIES" >&2
      sleep "$ROLLOUT_SEARCH_WAIT"
    else
      # Failed after all retries
      total_time=$((ROLLOUT_SEARCH_TRIES * ROLLOUT_SEARCH_WAIT))
      fail search "the engine did not start in $total_time s"
    fi
  else
    # No results and no starting marker: fail immediately
    fail search "/?q=satipatthana has no result list"
  fi
done

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

[ "$mode" = worker ] || exit 0

# ------------------------------------------------------------- hydrate (T3)

# The Chrome to drive: $CHROME, or the first one found.
chrome="${CHROME:-}"
if [ -n "$chrome" ]; then
  [ -x "$chrome" ] || { echo "rollout-check: CHROME is not an executable Chrome ($chrome)" >&2; exit 2; }
else
  for candidate in google-chrome google-chrome-stable chromium chromium-browser; do
    chrome="$(command -v "$candidate" 2>/dev/null)" && break
  done
  mac_chrome="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
  if [ -z "$chrome" ] && [ -x "$mac_chrome" ]; then chrome="$mac_chrome"; fi
  [ -n "$chrome" ] || { echo "rollout-check: no Chrome to check hydration with: set CHROME" >&2; exit 2; }
fi
limit="${ROLLOUT_CHROME_TIMEOUT:-60}"
case "$limit" in
  '' | *[!0-9]*) echo "rollout-check: ROLLOUT_CHROME_TIMEOUT must be a number of seconds" >&2; exit 2 ;;
esac

# limited CMD…: CMD, stopped after $limit seconds (macOS has no `timeout`).
limited() {
  if command -v timeout >/dev/null 2>&1; then
    timeout "$limit" "$@"
  else
    perl -e 'alarm shift; exec @ARGV' "$limit" "$@"
  fi
}

# hydrate PATH N: Chrome loads $url$PATH, its DOM in $work/dom-N, its
# console in $work/console-N; fails, or exits 3 on a challenge.
hydrate() {
  local path=$1 n=$2 rc=0 flags=()
  # A runner's Chrome has no usable sandbox; a laptop's keeps it.
  [ "${CI:-}" = true ] && flags+=(--no-sandbox)
  limited "$chrome" --headless=new --disable-gpu --no-first-run --no-default-browser-check \
    "--user-data-dir=$work/chrome-$n" "${flags[@]+"${flags[@]}"}" \
    --enable-logging=stderr --virtual-time-budget=15000 --dump-dom "$url$path" \
    > "$work/dom-$n" 2> "$work/console-$n" || rc=$?
  local panic
  panic="$(grep -aE -m 1 'panicked at|RuntimeError' "$work/console-$n" | cut -c 1-300)"
  [ -z "$panic" ] || fail hydrate "$path: the page panicked in the browser: $panic"
  if grep -aEq '_cf_chl_opt|<title>Just a moment' "$work/dom-$n"; then
    echo "rollout-check: challenge: Cloudflare challenged Chrome on $path; hydration was not checked (never a pass: run it by hand)" >&2
    exit 3
  fi
  if ! grep -aEq '<html[^>]*[[:space:]]data-hydrated' "$work/dom-$n"; then
    [ "$rc" = 0 ] || fail hydrate "$path: no data-hydrated on <html>: Chrome stopped (exit $rc, limit ${limit}s)"
    fail hydrate "$path: no data-hydrated on <html>: the page never hydrated"
  fi
}

n=0
for path in / '/?q=satipatthana' /privacy; do
  n=$((n + 1))
  hydrate "$path" "$n"
done
echo "rollout-check: hydrate ok"
