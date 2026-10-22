#!/usr/bin/env bash
# Tests for scripts/release-values.sh and its wiring in deploy.yml (task 028,
# the twin of learn-pali 060).
# Run: bash scripts/release-values.test.sh
#
# The script turns what Actions provides (GITHUB_REF_TYPE, GITHUB_REF_NAME,
# GITHUB_SHA) into the two values deploy.yml writes onto the Cloud Run
# revision: SEARCH_FRONT_ENV (the tag, main, or the branch) and
# SEARCH_FRONT_SHA (empty for a tag, sha7 otherwise), and compares them with a
# serving revision's env (`same`). The expected values below are written by
# hand, never computed with the script's own rules.
#
# What the wiring tests do NOT prove: the greps read deploy.yml's text, and
# the step cases run its scripts against a stub gcloud, so a real gcloud that
# behaves otherwise is not seen. A wrong script is caught by the literal-value
# tests, a wrong workflow (a value that never reaches the revision) only by
# the gate against the served page (rollout-check.sh, ROLLOUT_EXPECT_ENV and
# ROLLOUT_EXPECT_SHA, exit 8).
#
# Needs bash, jq, python3 with PyYAML. Exit 0 when every assertion held.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$here/.." && pwd)"
# Two hooks point the suite at a mutated copy (a local mutation run): the
# script, and the workflow. Set in CI they would make the suite test a copy,
# so they are refused there (exit 64).
for hook in RELEASE_TEST_SCRIPT RELEASE_TEST_WORKFLOW; do
  if [ -n "${CI:-}" ] && [ -n "${!hook:-}" ]; then
    echo "$(basename "$0"): the test hook $hook is refused when CI is set (it points the suite at a copy)" >&2
    exit 64
  fi
done
script="${RELEASE_TEST_SCRIPT:-$here/release-values.sh}"
workflow="${RELEASE_TEST_WORKFLOW:-$root/.github/workflows/deploy.yml}"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

checks=0
failed=0
pass() { checks=$((checks + 1)); echo "ok   - $1"; }
flunk() { checks=$((checks + 1)); failed=$((failed + 1)); echo "FAIL - $1"; }
assert_eq() { # name want got
  if [ "$2" = "$3" ]; then pass "$1"; else flunk "$1 (want '$2', got '$3')"; fi
}
assert_match() { # name pattern text
  if grep -Eq -- "$2" <<<"$3"; then pass "$1"; else flunk "$1 (no match for /$2/ in: $3)"; fi
}
assert_absent() { # name needle text
  if grep -Fq -- "$2" <<<"$3"; then flunk "$1 ('$2' found in: $3)"; else pass "$1"; fi
}
assert_file_has() { # name file pattern
  if grep -Eq -- "$3" "$2" 2>/dev/null; then pass "$1"; else flunk "$1 (/$3/ not in $2)"; fi
}
assert_file_lacks() { # name file pattern
  if grep -Eq -- "$3" "$2" 2>/dev/null; then flunk "$1 (/$3/ found in $2)"; else pass "$1"; fi
}

# rv TYPE NAME SHA [ARGS...] -> $out (stdout), $err (stderr), $code. The
# environment holds only those three variables, as an Actions step would.
rv() {
  local t=$1 n=$2 s=$3
  shift 3
  out="$(env -i PATH="$PATH" GITHUB_REF_TYPE="$t" GITHUB_REF_NAME="$n" GITHUB_SHA="$s" bash "$script" "$@" 2>"$work/err" </dev/null)"
  code=$?
  err="$(cat "$work/err")"
}
# refused NAME TYPE REF SHA: exit 1, a message on stderr, nothing on stdout, in
# every mode that reads the inputs.
refused() {
  local name=$1
  shift
  local mode
  for mode in "" env sha; do
    rv "$@" ${mode:+"$mode"}
    assert_eq "$name: exit 1 (mode '${mode:-default}')" "1" "$code"
    assert_eq "$name: nothing on stdout (mode '${mode:-default}')" "" "$out"
    if [ -n "$err" ]; then pass "$name: a message on stderr (mode '${mode:-default}')"; else flunk "$name: a message on stderr (mode '${mode:-default}')"; fi
  done
}

SHA=34c489c0123456789

echo "== release-values.sh: the values of the three events"
rv branch main "$SHA"
assert_eq "push to main: the exit is 0" "0" "$code"
assert_eq "push to main: the value line" "SEARCH_FRONT_ENV=main,SEARCH_FRONT_SHA=34c489c" "$out"
# A tag run by hand would be a workflow_dispatch event whose ref type is still
# `tag`: the event name must not change the values (this deploy.yml has no
# dispatch trigger, the script must not depend on that).
out="$(env -i PATH="$PATH" GITHUB_EVENT_NAME=workflow_dispatch GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.1.8 GITHUB_SHA="$SHA" bash "$script" 2>"$work/err" </dev/null)"
code=$?
assert_eq "a tag dispatched by hand: the exit is 0" "0" "$code"
assert_eq "a tag dispatched by hand: the value line (the tag, no sha)" "SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=" "$out"
out="$(env -i PATH="$PATH" GITHUB_EVENT_NAME=workflow_dispatch GITHUB_REF_TYPE=branch GITHUB_REF_NAME=feat/x GITHUB_SHA="$SHA" bash "$script" 2>"$work/err" </dev/null)"
assert_eq "a branch dispatched by hand: the value line (the branch and the sha7)" "SEARCH_FRONT_ENV=feat/x,SEARCH_FRONT_SHA=34c489c" "$out"
rv tag v0.1.8 "$SHA"
assert_eq "tag v0.1.8: the exit is 0" "0" "$code"
assert_eq "tag v0.1.8: the value line (no sha)" "SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=" "$out"
rv branch feat/x "$SHA"
assert_eq "dispatch of feat/x: the value line" "SEARCH_FRONT_ENV=feat/x,SEARCH_FRONT_SHA=34c489c" "$out"
rv tag v0.1.8 ""
assert_eq "tag with a blank GITHUB_SHA: still ok" "0" "$code"
assert_eq "tag with a blank GITHUB_SHA: the sha stays empty" "SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=" "$out"
rv tag v0.1.8 "zz"
assert_eq "tag: GITHUB_SHA is ignored even when it is garbage" "SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=" "$out"
rv branch main 34c489c
assert_eq "a branch with exactly 7 characters of sha" "SEARCH_FRONT_ENV=main,SEARCH_FRONT_SHA=34c489c" "$out"
rv branch main 34C489C0123
assert_eq "an upper-case hex sha is kept as given" "SEARCH_FRONT_ENV=main,SEARCH_FRONT_SHA=34C489C" "$out"

echo "== release-values.sh: the env and sha modes"
rv branch main "$SHA" env
assert_eq "env, main" "main" "$out"
rv branch main "$SHA" sha
assert_eq "sha, main" "34c489c" "$out"
rv tag v0.1.8 "$SHA" env
assert_eq "env, tag" "v0.1.8" "$out"
rv tag v0.1.8 "$SHA" sha
assert_eq "sha, tag: an empty value" "" "$out"
assert_eq "sha, tag: the exit is 0" "0" "$code"
assert_eq "sha, tag: one empty line" "1" "$(env -i PATH="$PATH" GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.1.8 GITHUB_SHA="$SHA" bash "$script" sha | grep -c '^$')"
rv branch feat/x "$SHA" env
assert_eq "env, dispatch" "feat/x" "$out"
rv branch feat/x "$SHA" sha
assert_eq "sha, dispatch" "34c489c" "$out"

echo "== release-values.sh: refusals (exit 1, stderr, empty stdout)"
refused "a pull_request ref type" pull_request main "$SHA"
refused "an empty ref type" "" main "$SHA"
refused "an empty ref name" branch "" "$SHA"
refused "an empty tag name" tag "" "$SHA"
refused "a name with a comma" branch "a,b" "$SHA"
refused "a tag name with a comma" tag "v1,2" "$SHA"
refused "a name with a space" branch "a b" "$SHA"
refused "a name with a tab" branch "$(printf 'a\tb')" "$SHA"
refused "a name with a newline" branch "$(printf 'a\nb')" "$SHA"
# 028 review F1: the env value, the meta the server escapes and the gate's raw
# compare agree only for [A-Za-z0-9._/-]; a legal git ref char outside it
# (&, >, ", =) would fail the gate forever (exit 8), so it is refused here.
refused "a name with an ampersand" branch "feat/a&b" "$SHA"
refused "a name with a greater-than" branch "feat/a>b" "$SHA"
refused "a name with a double quote" branch 'feat/a"b' "$SHA"
refused "a name with an equals sign" branch "feat/a=b" "$SHA"
refused "a tag name with an ampersand" tag "v1&2" "$SHA"
rv branch 'feat/a"b' "$SHA"
if [[ "$err" == *'feat/a"b'* ]]; then pass "a refused ref name is named on stderr"; else flunk "a refused ref name is named on stderr"; fi
rv branch "feat/a.b-c_d/e" "$SHA"
assert_eq "the whole allowed class [A-Za-z0-9._/-] is accepted: exit 0" "0" "$code"
assert_eq "the whole allowed class is accepted: the value line" "SEARCH_FRONT_ENV=feat/a.b-c_d/e,SEARCH_FRONT_SHA=34c489c" "$out"
rv tag "v0.1.8-rc.1" ""
assert_eq "a tag with a dot and a dash is accepted" "SEARCH_FRONT_ENV=v0.1.8-rc.1,SEARCH_FRONT_SHA=" "$out"
refused "a branch with a blank sha" branch main ""
refused "a branch with a 6-character sha" branch main 34c489
refused "a branch with a sha that is not hex" branch main zzzzzzzzz
out="$(env -i PATH="$PATH" bash "$script" 2>"$work/err" </dev/null)"
code=$?
assert_eq "unset variables: exit 1" "1" "$code"
assert_eq "unset variables: nothing on stdout" "" "$out"
assert_match "unset variables: a message" "." "$(cat "$work/err")"
out="$(env -i PATH="$PATH" GITHUB_REF_TYPE=tag bash "$script" 2>"$work/err" </dev/null)"
code=$?
assert_eq "a tag with no ref name variable: exit 1" "1" "$code"
out="$(env -i PATH="$PATH" GITHUB_REF_TYPE=branch GITHUB_REF_NAME=main bash "$script" 2>"$work/err" </dev/null)"
code=$?
assert_eq "a branch with no sha variable: exit 1" "1" "$code"
assert_eq "a branch with no sha variable: nothing on stdout" "" "$out"

echo "== release-values.sh: an unknown mode"
rv branch main "$SHA" nope
assert_eq "unknown mode: exit 64" "64" "$code"
assert_eq "unknown mode: nothing on stdout" "" "$out"
rv branch main "$SHA" env extra
assert_eq "two arguments: exit 64" "64" "$code"

echo "== release-values.sh same: the serving revision against the intended release"
# same TYPE NAME SHA JSON -> $code, $err
same() {
  local t=$1 n=$2 s=$3 json=$4
  out="$(printf '%s' "$json" | env -i PATH="$PATH" GITHUB_REF_TYPE="$t" GITHUB_REF_NAME="$n" GITHUB_SHA="$s" bash "$script" same 2>"$work/err")"
  code=$?
  err="$(cat "$work/err")"
}
rev() { # ENV-ARRAY-JSON -> a revision document holding it (hand-written shape of `gcloud run revisions describe --format json`)
  printf '{"metadata":{"name":"search-front-dev-00001-aaa"},"spec":{"containers":[{"image":"registry/search-front:h-1","env":%s}]}}' "$1"
}
NOENV='{"metadata":{"name":"r"},"spec":{"containers":[{"image":"registry/search-front:h-1"}]}}'
same branch main "$SHA" "$NOENV"
assert_eq "no env array at all vs a push to main: differs" "1" "$code"
assert_match "no env array vs main: 'differs:' on stderr" "^differs:" "$err"
assert_eq "no env array vs main: nothing on stdout" "" "$out"
same tag v0.1.8 "$SHA" "$NOENV"
assert_eq "no env array at all vs a tag: SEARCH_FRONT_ENV is absent, differs" "1" "$code"
same tag v0.1.8 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"}]')"
assert_eq "env v0.1.8, sha absent vs tag v0.1.8: same" "0" "$code"
same tag v0.1.8 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"},{"name":"SEARCH_FRONT_SHA","value":""}]')"
assert_eq "env v0.1.8, sha empty vs tag v0.1.8: same" "0" "$code"
same tag v0.1.8 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_SHA","value":""},{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"}]')"
assert_eq "the order of the entries does not matter" "0" "$code"
same tag v0.1.8 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"},{"name":"SEARCH_FRONT_SHA"}]')"
assert_eq "an entry with no value key is an empty value: same" "0" "$code"
same tag v0.1.8 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"},{"name":"SEARCH_FRONT_SHA","value":"34c489c"}]')"
assert_eq "tag intended, sha present on the revision: differs" "1" "$code"
same branch main "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"34c489c"}]')"
assert_eq "env main sha 34c489c vs the intended main 34c489c: same" "0" "$code"
same branch main 5555555aaaa "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"34c489c"}]')"
assert_eq "env main sha 34c489c vs a new push 5555555: differs" "1" "$code"
assert_match "a new push: the message names the sha" "SEARCH_FRONT_SHA.*34c489c.*5555555" "$err"
same tag v0.1.9 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"34c489c"}]')"
assert_eq "env main vs a tag: differs" "1" "$code"
same tag v0.1.9 "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"},{"name":"SEARCH_FRONT_SHA","value":""}]')"
assert_eq "tag v0.1.8 on the revision vs v0.1.9 intended: differs" "1" "$code"
same branch main "$SHA" "$(rev '[{"name":"RUST_LOG","value":"info"}]')"
assert_eq "an unrelated env array (RUST_LOG only) vs a branch: differs" "1" "$code"
same branch main "$SHA" "$(rev '[{"name":"RUST_LOG","value":"info"},{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"34c489c"}]')"
assert_eq "other entries beside the two do not matter: same" "0" "$code"
same branch main "$SHA" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"34c489c"}]' | sed 's/"env"/"nv"/')"
assert_eq "a document with no env key: differs, not an error" "1" "$code"
same branch main "$SHA" '{"spec":{"containers":[]}}'
assert_eq "a revision with no container: differs, not an error" "1" "$code"
# JSON that parses but whose env is not a list of entries (a
# string, an object, numbers) is a revision this script cannot read: exit 2
# with a message, never "differs" (deploy.yml logs the two cases apart).
for bad in '"oops"' '{"name":"SEARCH_FRONT_ENV"}' '[1,2]'; do
  same branch main "$SHA" "$(rev "$bad")"
  assert_eq "env $bad (not a list of entries): exit 2" "2" "$code"
  assert_match "env $bad: the message says the env is not a list" "env .*not a list" "$err"
  assert_eq "env $bad: nothing on stdout" "" "$out"
done
same branch main "$SHA" '{"spec":{"containers":"oops"}}'
assert_eq "containers that is not a list: exit 2" "2" "$code"
same branch main "$SHA" 'not json {'
assert_eq "invalid JSON: exit 2" "2" "$code"
assert_eq "invalid JSON: nothing on stdout" "" "$out"
same branch main "$SHA" ''
assert_eq "empty stdin: exit 2" "2" "$code"
same branch main "" "$(rev '[{"name":"SEARCH_FRONT_ENV","value":"main"}]')"
assert_eq "same: bad inputs are refused, exit 1" "1" "$code"

echo "== deploy.yml: the wiring (a text read, see the header for what it does not prove)"
assert_eq "release-values.sh is executable" "yes" "$([ -x "$script" ] && echo yes || echo no)"
assert_eq "release-values.sh starts with the bash shebang" "#!/usr/bin/env bash" "$(head -n 1 "$script")"
ROLLOUT_STEP="Roll out without traffic, check, then serve"
step_run() { # KEY VALUE -> the script of the step whose KEY (id | name) is VALUE
  python3 - "$workflow" "$1" "$2" <<'PY'
import sys, yaml
for s in yaml.safe_load(open(sys.argv[1]))["jobs"]["site"]["steps"]:
    if s.get(sys.argv[2]) == sys.argv[3]:
        print(s.get("run", ""))
        break
else:
    sys.exit(3)
PY
}
step_run id current > "$work/step-current.sh" 2>/dev/null || { flunk "deploy.yml has the step 'current'"; : > "$work/step-current.sh"; }
step_run name "$ROLLOUT_STEP" > "$work/step-rollout.sh" 2>/dev/null || { flunk "deploy.yml has the step '$ROLLOUT_STEP'"; : > "$work/step-rollout.sh"; }
assert_file_has "the rollout step writes the release with --update-env-vars" "$work/step-rollout.sh" '--update-env-vars "\$\(scripts/release-values\.sh\)"'
assert_file_has "the rollout step keeps --no-traffic and the tag" "$work/step-rollout.sh" '--no-traffic --tag'
assert_file_has "the rollout step still moves the traffic with --to-latest (this repository's move)" "$work/step-rollout.sh" 'update-traffic .*--to-latest'
assert_file_lacks "the set variant of the flag appears nowhere (it would wipe RUST_LOG, SEARCH_API_URL and the secret reference)" "$workflow" '--set-env-vars'
assert_file_lacks "no BUILD_BRANCH or BUILD_SHA build arg is left" "$workflow" 'BUILD_(BRANCH|SHA)'
assert_file_lacks "no build-args is left" "$workflow" 'build-args:'
assert_file_has "the current step reads the revision once, as JSON" "$work/step-current.sh" 'revisions describe "\$SERVING".*--format json'
assert_file_has "the current step asks 'release-values.sh same' about the serving revision" "$work/step-current.sh" 'release-values\.sh same <<<"\$REVISION_JSON" \|\| RELEASE_RC=\$\?'
assert_file_lacks "the current step holds no SAME_RC (one handler of the exit code, RELEASE_RC)" "$work/step-current.sh" 'SAME_RC'
assert_file_has "the rollout step expects the env" "$work/step-rollout.sh" 'ROLLOUT_EXPECT_ENV="\$\(scripts/release-values\.sh env\)"'
assert_file_has "the rollout step expects the sha" "$work/step-rollout.sh" 'ROLLOUT_EXPECT_SHA="\$\(scripts/release-values\.sh sha\)"'
assert_file_has "the rollout step exports both" "$work/step-rollout.sh" 'export ROLLOUT_EXPECT_ENV ROLLOUT_EXPECT_SHA'
assert_file_has "the rollout step has its own arm for exit 8" "$work/step-rollout.sh" '"\$RC" = 8'
assert_file_lacks "this repository's deploy.yml gets no workflow_dispatch trigger (a branch dispatch does not exist here)" "$workflow" 'workflow_dispatch'

# ---- the steps, run with a stub gcloud and a stub rollout-check
stub="$work/stub"
proj="$work/proj"
mkdir -p "$stub/bin" "$proj/scripts"
cat > "$stub/bin/gcloud" <<'SH'
#!/usr/bin/env bash
# A stub gcloud over the state in $STUB (the arguments are logged, the secret never is).
echo "gcloud $*" >> "$STUB/calls"
all="$*"
case "$1 $2 $3" in
  "run services describe")
    # The service: the traffic entry at 100% names the serving revision (or
    # follows the latest), and the tagged entry carries the rollout's URL.
    serving=$(cat "$STUB/serving")
    if [ -f "$STUB/follow-latest" ]; then entry='{"percent":100,"latestRevision":true}'; else entry="{\"percent\":100,\"revisionName\":\"$serving\"}"; fi
    if [ -f "$STUB/no-traffic" ]; then entry='{"percent":50,"revisionName":"a"},{"percent":50,"revisionName":"b"}'; fi
    printf '{"status":{"url":"https://search-front-dev-x.a.run.app","latestReadyRevisionName":"%s","latestCreatedRevisionName":"%s","traffic":[%s,{"tag":"sha-abc1234","percent":0,"revisionName":"search-front-dev-00002-bbb","url":"https://sha-abc1234---x.a.run.app"}]}}' \
      "$(cat "$STUB/latest-ready")" "$(cat "$STUB/latest-ready")" "$entry"
    ;;
  "run revisions describe")
    # `--format json` only: the image and the env come from the same document.
    case "$all" in *"--format json"*) ;; *) echo "stub gcloud: revisions describe needs --format json" >&2; exit 97 ;; esac
    # raw-<rev>: the answer verbatim (a malformed or image-less revision).
    if [ -f "$STUB/raw-$4" ]; then cat "$STUB/raw-$4"; exit 0; fi
    img=$(cat "$STUB/image-$4")
    if [ -f "$STUB/env-$4" ]; then
      printf '{"spec":{"containers":[{"image":"%s","env":%s}]}}' "$img" "$(cat "$STUB/env-$4")"
    else
      printf '{"spec":{"containers":[{"image":"%s"}]}}' "$img"
    fi ;;
  "run deploy"*) ;;
  "run services update-traffic") ;;
  "secrets versions access") cat "$STUB/secret-value" ;;
  *) echo "stub gcloud: unexpected: $*" >&2; exit 99 ;;
esac
SH
cat > "$proj/scripts/rollout-check.sh" <<'SH'
#!/usr/bin/env bash
# A stub: logs its arguments, the header file's mode and the release it is
# given, exits with the code in $STUB/rc.
echo "rollout-check $*" >> "$STUB/calls"
echo "rollout-check-expect env=${ROLLOUT_EXPECT_ENV-<unset>} sha=${ROLLOUT_EXPECT_SHA-<unset>}" >> "$STUB/calls"
[ -f "$2" ] && echo "header-file $(ls -l "$2" | cut -c1-10)" >> "$STUB/calls"
echo "stub rollout-check"
exit "$(cat "$STUB/rc" 2>/dev/null || echo 0)"
SH
cp "$script" "$proj/scripts/release-values.sh"
cp "$here/serving-revision.sh" "$proj/scripts/serving-revision.sh"
chmod +x "$stub/bin/gcloud" "$proj/scripts/"*.sh

SECRET="s3cr3t-origin-value-0123456789"
reset_stub() { # image-of-the-serving-revision
  rm -rf "$stub/state"
  mkdir -p "$stub/state"
  export STUB="$stub/state"
  printf 'search-front-dev-00001-aaa' > "$STUB/serving"
  printf 'search-front-dev-00002-bbb' > "$STUB/latest-ready"
  printf '%s' "$1" > "$STUB/image-search-front-dev-00001-aaa"
  printf 'registry/search-front:h-new' > "$STUB/image-search-front-dev-00002-bbb"
  printf '%s' "$SECRET" > "$STUB/secret-value"
  : > "$STUB/calls"
  : > "$STUB/out"
  rm -f "$RUNNER_TEMP/origin-header"
}
export RUNNER_TEMP="$work/runner-temp"
mkdir -p "$RUNNER_TEMP"

# step NAME [VAR=value ...]: runs the extracted step as the runner would. The
# default ref is a push to main of abc1234def5678.
step() {
  local name=$1
  shift
  (
    cd "$proj" || exit 99
    env PATH="$stub/bin:$PATH" STUB="$STUB" GITHUB_OUTPUT="$STUB/out" RUNNER_TEMP="$RUNNER_TEMP" \
      ENV=dev REGION=europe-west1 PROJECT=proj GITHUB_SHA=abc1234def5678 \
      GITHUB_REF_TYPE=branch GITHUB_REF_NAME=main \
      IMAGE_HASH=registry/search-front:h-new "$@" \
      bash "$work/step-$name.sh"
  ) 2>&1
}
calls() { cat "$STUB/calls"; }
output() { sed -n "s/^$1=//p" "$STUB/out" | head -n 1; }

# The env array of a revision that serves the push to main of abc1234def5678.
MAIN_ENV='[{"name":"RUST_LOG","value":"info"},{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"abc1234"}]'
echo "== deploy.yml, step 'current': the skip needs the image AND the release"
reset_stub "registry/search-front:h-old"
out="$(step current)"
assert_eq "current: PREV is the serving revision" "search-front-dev-00001-aaa" "$(output serving)"
assert_eq "current: another image is running, so deploy" "false" "$(output running)"
reset_stub "registry/search-front:h-new"
printf '%s' "$MAIN_ENV" > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current)"
assert_eq "current (same image, same release): the skip" "true" "$(output running)"
assert_eq "current (same image, same release): serving is PREV" "search-front-dev-00001-aaa" "$(output serving)"
assert_absent "current (same image, same release): nothing is deployed" "run deploy" "$(calls)"
assert_match "current (same image, same release): says nothing to deploy" "nothing to deploy" "$out"
reset_stub "registry/search-front:h-new"
printf '%s' '[{"name":"SEARCH_FRONT_ENV","value":"main"},{"name":"SEARCH_FRONT_SHA","value":"1111111"}]' > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current)"
assert_eq "current (same image, another push): a new revision is wanted" "false" "$(output running)"
assert_eq "current (same image, another push): PREV is still the serving revision" "search-front-dev-00001-aaa" "$(output serving)"
assert_match "current (same image, another push): the message says another release" "serves the image but another release .*new revision with the new env" "$out"
assert_absent "current (same image, another push): not the skip message" "nothing to deploy" "$out"
# The image is the intended one AND the sha7 is the intended one, but the
# revision was rolled out for another ref: the push to main is still another
# release, so a new revision.
reset_stub "registry/search-front:h-new"
printf '%s' '[{"name":"SEARCH_FRONT_ENV","value":"feat/x"},{"name":"SEARCH_FRONT_SHA","value":"abc1234"}]' > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current)"
assert_eq "current (same image, another ref's env, same sha7): a new revision is wanted" "false" "$(output running)"
assert_match "current (same image, another ref's env, same sha7): the message says another release" "serves the image but another release" "$out"
reset_stub "registry/search-front:h-new"
out="$(step current)"
assert_eq "current (same image, no env on the revision): a new revision is wanted" "false" "$(output running)"
reset_stub "registry/search-front:h-new"
printf '%s' '[{"name":"RUST_LOG","value":"info"}]' > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current)"
assert_eq "current (same image, an env array with RUST_LOG only): a new revision is wanted" "false" "$(output running)"
# A tag: absent sha == empty sha, so v0.1.8 already served is a skip; a new tag
# v0.1.9 on the image main built still deploys.
reset_stub "registry/search-front:h-new"
printf '%s' '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"}]' > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.1.8)"
assert_eq "current (tag, env equal, sha absent): absent == empty, so skip" "true" "$(output running)"
reset_stub "registry/search-front:h-new"
printf '%s' '[{"name":"SEARCH_FRONT_ENV","value":"v0.1.8"},{"name":"SEARCH_FRONT_SHA","value":""}]' > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.1.8)"
assert_eq "current (tag, env equal, sha empty): skip" "true" "$(output running)"
reset_stub "registry/search-front:h-new"
printf '%s' "$MAIN_ENV" > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.1.9 ENV=prod)"
assert_eq "current (a new tag on main's image): a new revision" "false" "$(output running)"
reset_stub "registry/search-front:h-new"
out="$(step current GITHUB_REF_TYPE=branch GITHUB_REF_NAME=feat/x GITHUB_SHA=zz)"
rc=$?
assert_eq "current: inputs release-values.sh refuses fail the step" "1" "$rc"
# A serving revision whose JSON cannot be read, or that names no image, never
# kills the step and never skips: it falls open to a deploy and the log says
# why.
reset_stub "registry/search-front:h-new"
printf 'not json' > "$STUB/raw-search-front-dev-00001-aaa"
out="$(step current)"
rc=$?
assert_eq "current (revision JSON malformed): the step still exits 0" "0" "$rc"
assert_eq "current (revision JSON malformed): a new revision is wanted" "false" "$(output running)"
assert_eq "current (revision JSON malformed): PREV is still the serving revision" "search-front-dev-00001-aaa" "$(output serving)"
assert_match "current (revision JSON malformed): the log names the reason" "could not read the serving revision's image/release \\(.*\\): deploying a new revision" "$out"
reset_stub "registry/search-front:h-new"
printf '%s' '{"spec":{"containers":[{}]}}' > "$STUB/raw-search-front-dev-00001-aaa"
out="$(step current)"
rc=$?
assert_eq "current (revision names no image): the step still exits 0" "0" "$rc"
assert_eq "current (revision names no image): a new revision is wanted" "false" "$(output running)"
assert_match "current (revision names no image): the log names the reason" "could not read the serving revision's image/release \\(.*image.*\\): deploying a new revision" "$out"
reset_stub "registry/search-front:h-new"
printf '%s' '{"spec":{"containers":[]}}' > "$STUB/raw-search-front-dev-00001-aaa"
out="$(step current)"
rc=$?
assert_eq "current (revision with no container): the step still exits 0" "0" "$rc"
assert_eq "current (revision with no container): a new revision is wanted" "false" "$(output running)"
# The image is the one to run, but the revision's env is of the wrong type (a
# string, not a list): `release-values.sh same` cannot read it (exit 2). The
# step deploys (fail open), says the release could not be read in its own
# line, and never claims "another release" (that is the exit 1 case above).
reset_stub "registry/search-front:h-new"
printf '%s' '{"spec":{"containers":[{"image":"registry/search-front:h-new","env":"oops"}]}}' > "$STUB/raw-search-front-dev-00001-aaa"
out="$(step current)"
rc=$?
assert_eq "current (good image, env of the wrong type): the step still exits 0" "0" "$rc"
assert_eq "current (good image, env of the wrong type): a new revision is wanted" "false" "$(output running)"
assert_eq "current (good image, env of the wrong type): PREV is still the serving revision" "search-front-dev-00001-aaa" "$(output serving)"
assert_match "current (good image, env of the wrong type): its own line says the release could not be read" "could not read the serving revision's release.*: deploying a new revision" "$out"
assert_absent "current (good image, env of the wrong type): never 'another release'" "another release" "$out"
# The happy skip stays a skip (the pair of the case above).
reset_stub "registry/search-front:h-new"
printf '%s' "$MAIN_ENV" > "$STUB/env-search-front-dev-00001-aaa"
out="$(step current)"
assert_eq "current (readable, same image and release): still a skip" "true" "$(output running)"
assert_absent "current (readable, same image and release): no could-not-read message" "could not read" "$out"
# This step's own contract, kept: the 100% entry that follows the latest
# revision names the latest ready one, and no revision at 100% fails the job.
reset_stub "registry/search-front:h-old"
touch "$STUB/follow-latest"
printf 'registry/search-front:h-old' > "$STUB/image-search-front-dev-00002-bbb"
out="$(step current)"
assert_eq "current: a service that follows the latest revision names its ready one" "search-front-dev-00002-bbb" "$(output serving)"
reset_stub "registry/search-front:h-old"
touch "$STUB/no-traffic"
out="$(step current)"
rc=$?
assert_eq "current: no revision at 100% fails the step" "1" "$rc"
assert_match "current: no revision at 100%: the error names the rollback" "no revision serves search-front-dev: the rollback would have nowhere to go" "$out"

echo "== deploy.yml, step '$ROLLOUT_STEP': the release goes onto the revision and the gate reads it"
reset_stub "registry/search-front:h-old"
out="$(step rollout)"
rc=$?
assert_eq "rollout (push to main): the step exits 0" "0" "$rc"
assert_match "rollout (push to main): the deploy writes the release with --update-env-vars" "run deploy search-front-dev .*--image registry/search-front:h-new .*--update-env-vars SEARCH_FRONT_ENV=main,SEARCH_FRONT_SHA=abc1234" "$(calls)"
assert_match "rollout (push to main): the deploy has no traffic and the sha tag" "--no-traffic --tag sha-abc1234" "$(calls)"
assert_match "rollout (push to main): the gate is given the URL and the header file" "rollout-check https://sha-abc1234---x.a.run.app $RUNNER_TEMP/origin-header" "$(calls)"
assert_match "rollout (push to main): the gate expects env main and sha abc1234" "rollout-check-expect env=main sha=abc1234" "$(calls)"
assert_match "rollout (push to main): the header file is 0600" "header-file -rw-------" "$(calls)"
assert_match "rollout (push to main): then the traffic moves to the latest" "update-traffic search-front-dev .*--to-latest" "$(calls)"
# The only line that may hold the value is the mask command itself.
assert_absent "rollout (push to main): the origin secret is not logged" "$SECRET" "$(calls)$(grep -v '^::add-mask::' <<<"$out")"
assert_match "rollout (push to main): the secret is masked before any later line" "^::add-mask::" "$out"
assert_absent "rollout (push to main): the set variant of the flag is never called" "--set-env-vars" "$(calls)"
reset_stub "registry/search-front:h-old"
out="$(step rollout GITHUB_REF_TYPE=tag GITHUB_REF_NAME=v0.1.8 ENV=prod)"
rc=$?
assert_eq "rollout (tag v0.1.8): the step exits 0" "0" "$rc"
assert_match "rollout (tag v0.1.8): the deploy writes the tag and an empty sha" "run deploy search-front-prod .*--update-env-vars SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=( |$)" "$(calls)"
assert_match "rollout (tag v0.1.8): the gate expects env v0.1.8 and an EMPTY sha, set (not unset)" "rollout-check-expect env=v0.1.8 sha=$" "$(calls)"
reset_stub "registry/search-front:h-old"
printf '%s' 8 > "$STUB/rc"
out="$(step rollout)"
rc=$?
assert_eq "rollout (gate exit 8): the step fails" "1" "$rc"
assert_match "rollout (gate exit 8): the error says another release than the one deployed" "::error::the revision at its tagged URL serves another release than the one deployed" "$out"
assert_match "rollout (gate exit 8): the error names the release expected" "env 'main', sha 'abc1234'" "$out"
assert_match "rollout (gate exit 8): the traffic is untouched" "the traffic is untouched" "$out"
assert_absent "rollout (gate exit 8): the traffic does not move" "update-traffic" "$(calls)"
reset_stub "registry/search-front:h-old"
printf '%s' 1 > "$STUB/rc"
out="$(step rollout)"
rc=$?
assert_eq "rollout (gate exit 1): the step fails" "1" "$rc"
assert_match "rollout (gate exit 1): its own message, not the release one" "::error::the revision at its tagged URL failed rollout-check.sh; the traffic is untouched" "$out"
assert_absent "rollout (gate exit 1): not the release message" "another release" "$out"
assert_absent "rollout (gate exit 1): the traffic does not move" "update-traffic" "$(calls)"
# A ref release-values.sh refuses never reaches this step: step 'current' runs
# the script first and fails the job (tested above). Inside this step the
# command substitution of --update-env-vars cannot stop the deploy by itself
# (set -e ignores a failing substitution in an argument), so that guard is the
# earlier step's, not this one's.

echo "== ci.yml: every test this repository holds for the release runs in CI"
# A text read of the run: lines. It does not catch a step that is present but
# skipped by an `if:`, or placed in a job that never runs.
ci_runs() { # -> the run: lines of ci.yml
  python3 -c '
import sys, yaml
for job in yaml.safe_load(open(sys.argv[1]))["jobs"].values():
    for step in job.get("steps", []):
        if "run" in step:
            print(step["run"])
' "$root/.github/workflows/ci.yml"
}
ci_runs > "$work/ci-runs"
assert_file_has "ci.yml runs release-values.test.sh" "$work/ci-runs" '^bash scripts/release-values\.test\.sh$'
assert_file_has "ci.yml runs lifecycle.test.sh (written by lane C)" "$work/ci-runs" '^bash infra/scripts/lifecycle\.test\.sh$'
assert_file_has "ci.yml runs the release_document browser test with chromedriver" "$work/ci-runs" 'CHROMEDRIVER="\$CHROMEWEBDRIVER/chromedriver" cargo test --locked -p search_front --no-default-features --features csr --target wasm32-unknown-unknown --test release_document$'
# The browser test needs the wasm-bindgen runner: its step carries the env.
assert_eq "the release_document step sets the wasm-bindgen runner" "wasm-bindgen-test-runner" "$(python3 - "$root/.github/workflows/ci.yml" <<'PY'
import sys, yaml
for job in yaml.safe_load(open(sys.argv[1]))["jobs"].values():
    for step in job.get("steps", []):
        if step.get("run", "").endswith("--test release_document"):
            print((step.get("env") or {}).get("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER", ""))
PY
)"

echo "== the test hooks are refused when CI is set"
# A hook points the suite at a mutated copy. A leftover in the environment of
# a CI job would make the suite test the copy, not the code: the suite exits
# 64 and names the hook instead. Each hook is set to the real file it
# replaces, so the only reason to refuse is CI. RELEASE_TEST_NESTED stops the
# inner run of this very suite from running this section again.
if [ -z "${RELEASE_TEST_NESTED:-}" ]; then
  suite="$here/release-values.test.sh"
  hook_refused() { # HOOK VALUE
    local hook=$1 value=$2 hout hcode
    hout="$(env CI=1 RELEASE_TEST_NESTED=1 "$hook=$value" bash "$suite" 2>&1 </dev/null)"
    hcode=$?
    assert_eq "CI=1 and $hook set: the suite exits 64" "64" "$hcode"
    assert_match "CI=1 and $hook set: the message names the hook" "$hook" "$hout"
    assert_eq "CI=1 and $hook set: the suite ran no check" "0" "$(grep -c '^ok ' <<<"$hout")"
  }
  hook_refused RELEASE_TEST_SCRIPT "$here/release-values.sh"
  hook_refused RELEASE_TEST_WORKFLOW "$root/.github/workflows/deploy.yml"
  # And the pair: with no CI, a hook is still honoured (a mutation run works).
  hout="$(env -u CI RELEASE_TEST_NESTED=1 RELEASE_TEST_SCRIPT="$here/release-values.sh" bash "$suite" 2>&1 </dev/null)"
  assert_eq "no CI and RELEASE_TEST_SCRIPT set: the suite runs and passes" "0" "$?"
  assert_match "no CI and RELEASE_TEST_SCRIPT set: the suite ran its checks" "^ok " "$hout"
fi

echo "== $checks checks, $failed failed"
[ "$failed" = 0 ]
