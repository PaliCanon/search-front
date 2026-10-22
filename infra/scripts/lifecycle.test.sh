#!/usr/bin/env bash
# Tests for the Cloud Run service's lifecycle (task 028): tofu must ignore the
# container env, because deploy.yml writes SEARCH_FRONT_ENV and SEARCH_FRONT_SHA
# with `gcloud run deploy --update-env-vars`; a later `tofu apply` that rewrote
# the template would flip the badge back to "dev" with no diff shown. The list
# is read on the `ignore_changes = [` line itself, never from comments, and the
# README must carry the apply order (the ignore is applied BEFORE the first
# deploy that writes the env).
# Not caught: an `env` entry on that line that sits in a different lifecycle
# block (the file holds one such line today, asserted below), or a README
# sentence that says the order but is wrong about it (only its wording is read).
# Run: bash infra/scripts/lifecycle.test.sh
set -uo pipefail

here=$(cd "$(dirname "$0")" && pwd)
gcp="${GCP_TF:-$here/../gcp.tf}"
readme="${README_MD:-$here/../README.md}"
# A hook is a local mutation tool: set in CI it would make this suite test a
# copy, so it is refused (028).
for hook in GCP_TF README_MD; do
  if [ -n "${CI:-}" ] && [ -n "${!hook:-}" ]; then
    echo "$(basename "$0"): the test hook $hook is refused when CI is set (it points the suite at a copy)" >&2
    exit 64
  fi
done

pass=0
fail=0
ok() { pass=$((pass + 1)); echo "ok   $1"; }
ko() { fail=$((fail + 1)); echo "FAIL $1"; }
check() { # check "name" command...   (passes when the command succeeds)
  local name=$1
  shift
  if "$@"; then ok "$name"; else ko "$name"; fi
}

lines=$(grep -E '^[[:space:]]*ignore_changes[[:space:]]*=[[:space:]]*\[' "$gcp")
check "gcp.tf: exactly one ignore_changes list" test "$(printf '%s\n' "$lines" | grep -c .)" -eq 1

# The entries of the list, one per line, trimmed.
entries=$(printf '%s\n' "$lines" | sed -e 's/^[^=]*=[[:space:]]*\[//' -e 's/\][[:space:]]*$//' | tr ',' '\n' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')
has_entry() { printf '%s\n' "$entries" | grep -Fxq -- "$1"; }

for entry in \
  'template[0].containers[0].image' \
  'template[0].revision' \
  'scaling' \
  'client' \
  'client_version' \
  'template[0].containers[0].env'; do
  check "gcp.tf: ignore_changes holds $entry" has_entry "$entry"
done

# The brief of rollout-check.test.sh reads the first line of the list: it stays on one line.
check "gcp.tf: the list is closed on its own line" bash -c '[[ "$1" =~ \][[:space:]]*$ ]]' _ "$lines"

# The order is written above the list too.
comment=$(grep -B12 -E '^[[:space:]]*ignore_changes[[:space:]]*=[[:space:]]*\[' "$gcp" | grep -E '^[[:space:]]*#')
check "gcp.tf: the comment says the ignore is applied before the first deploy that writes the env" \
  bash -c '[[ "$1" =~ [Aa]pplied.*before\ the\ first\ deploy ]]' _ "$(printf '%s ' $comment | tr -s ' ')"

# README: the paragraph that says what tofu ignores.
para=$(awk '/^CI owns the image/{p=1} p&&/^$/{exit} p' "$readme" | tr '\n' ' ' | tr -s ' ')
check "README: the 'CI owns the image' paragraph is found" test -n "$para"
check "README: the ignored list names the container env" \
  bash -c '[[ "$1" =~ tofu\ ignores\ .*\`env\` ]]' _ "$para"
check "README: it says the apply must come before the first deploy that writes the env" \
  bash -c '[[ "$1" =~ applied.*before\ the\ first\ deploy ]]' _ "$para"
check "README: it names SEARCH_FRONT_ENV and SEARCH_FRONT_SHA" \
  bash -c '[[ "$1" == *SEARCH_FRONT_ENV* && "$1" == *SEARCH_FRONT_SHA* ]]' _ "$para"
check "README: it says how to change RUST_LOG or SEARCH_API_TOKEN afterwards" \
  bash -c '[[ "$1" == *RUST_LOG* && "$1" == *SEARCH_API_TOKEN* && "$1" == *-replace* ]]' _ "$para"
check "README: the deploy uses --update-env-vars and never --set-env-vars" \
  bash -c '[[ "$1" == *--update-env-vars*and\ never*--set-env-vars* ]]' _ "$para"
# After a `tofu apply -replace` the release env is gone: the README says so and
# says what restores it.
check "README: a replace of the service loses the release env until the next deploy" \
  bash -c '[[ "$1" == *"until the next deploy"* && "$1" == *SEARCH_FRONT_ENV* ]]' _ "$para"
# deploy.yml has no workflow_dispatch here (a push to main or a v* tag are its
# only triggers): the README must not promise a manual run or a branch dispatch,
# and must say what restores the release (the next push to main, or for a prod
# tag a by-hand gcloud deploy that writes the two variables, or a re-pushed tag).
has() { bash -c '[[ "$1" == *"$2"* ]]' _ "$1" "$2"; }
check "README: the next push to main writes main and the sha7" \
  has "$para" '`main · <sha7>`'
check "README: a prod tag is restored by a by-hand gcloud deploy with --update-env-vars" \
  has "$para" '`gcloud run deploy ... --update-env-vars SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=`'
check "README: no manual workflow run and no branch dispatch is promised (there is none)" \
  bash -c '[[ "$1" != *"gh workflow run"* && "$1" != *dispatch* ]]' _ "$para"

echo
echo "lifecycle.test.sh: $pass passed, $fail failed"
test "$fail" -eq 0
