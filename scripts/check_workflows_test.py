"""Tests for scripts/check_workflows.py (task 007, slice S5).

Each rule has a workflow that breaks it (the checker names the rule) and the
real workflows pass. Run: python3 scripts/check_workflows_test.py
"""

import pathlib
import re
import sys
import unittest

import yaml

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import check_workflows as cw  # noqa: E402

GOOD_DEPLOY = """
name: deploy
on:
  push:
    branches: [main]
concurrency:
  group: deploy-${{ startsWith(github.ref, 'refs/tags/') && 'prod' || 'dev' }}
  cancel-in-progress: false
permissions:
  contents: read
  id-token: write
jobs:
  site:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
        with:
          persist-credentials: false
      - uses: docker/build-push-action@v6
        with:
          push: true
          build-args: |
            BUILD_BRANCH=${{ github.ref_name }}
            BUILD_SHA=${{ github.sha }}
      - name: current
        id: current
        run: |
          set -euo pipefail
          REV=$(gcloud run services describe svc --format json | scripts/serving-revision.sh) || { echo "::error::no revision serves svc"; exit 1; }
          CURRENT=$(gcloud run revisions describe "$REV" --format='value(spec.containers[0].image)')
          echo "serving=$REV" >> "$GITHUB_OUTPUT"
          echo "running=$([ "$CURRENT" = "$IMAGE_HASH" ] && echo true || echo false)" >> "$GITHUB_OUTPUT"
      - name: rollout
        run: scripts/rollout-check.sh "$URL" "$HEADER_FILE"
      - name: traffic
        run: gcloud run services update-traffic svc --to-latest
      - name: public
        env:
          PREVIOUS: ${{ steps.current.outputs.serving }}
        run: |
          rollback() {
            gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100" || echo "::error::rollback to $PREVIOUS failed"
            NOW=$(gcloud run services describe svc --format json | scripts/serving-revision.sh) || NOW=""
            if [ "$NOW" != "$PREVIOUS" ]; then
              echo "::error::rollback did not restore $PREVIOUS"
              exit 5
            fi
          }
          rc=0
          scripts/rollout-check.sh --via-worker "https://$HOST" || rc=$?
          if [ "$rc" = 3 ]; then
            echo "::warning::challenged"
          elif [ "$rc" = 4 ]; then
            echo "::error::the Worker answered 500 — run infra/scripts/origin-secret push $ENV"
            exit "$rc"
          elif [ "$rc" != 0 ]; then
            if [ "$rc" = 1 ]; then rollback; fi
            exit "$rc"
          fi
          rc=0
          scripts/rollout-check.sh --refused "$SERVICE_URL" || rc=$?
          if [ "$rc" != 0 ]; then
            echo "::error::the bare origin answers — the service's ORIGIN_SECRET env is missing or empty (infra: the secret reference); traffic left as is"
            exit "$rc"
          fi
"""


def parse(text):
    return yaml.safe_load(text)


def violations(deploy=GOOD_DEPLOY, extra=None, actions=None):
    workflows = {"deploy.yml": parse(deploy)}
    for name, text in (extra or {}).items():
        workflows[name] = parse(text)
    return cw.check(workflows, {n: parse(t) for n, t in (actions or {}).items()})


def mutate(old, new):
    assert old in GOOD_DEPLOY, old
    return GOOD_DEPLOY.replace(old, new)


class Rules(unittest.TestCase):
    def test_a_good_deploy_passes(self):
        self.assertEqual(violations(), [])

    def test_a_secret_in_a_run_line_is_refused(self):
        bad = mutate('run: scripts/rollout-check.sh "$URL" "$HEADER_FILE"',
                     'run: \'curl -H "x-t: ${{ secrets.TOKEN }}" "$URL"\'')
        self.assertTrue(any("secret in a run" in v for v in violations(bad)))

    def test_a_secret_in_a_with_is_allowed(self):
        ok = mutate("push: true", "push: true\n          token: ${{ secrets.TOKEN }}")
        self.assertEqual(violations(ok), [])

    def test_a_write_permission_other_than_id_token_is_refused(self):
        bad = mutate("contents: read", "contents: write")
        self.assertTrue(any("permissions" in v for v in violations(bad)))

    def test_missing_permissions_are_refused(self):
        bad = mutate("permissions:\n  contents: read\n  id-token: write\n", "")
        self.assertTrue(any("permissions" in v for v in violations(bad)))

    def test_a_concurrency_group_per_env_is_required(self):
        bad = mutate("group: deploy-${{ startsWith(github.ref, 'refs/tags/') && 'prod' || 'dev' }}",
                     "group: deploy")
        self.assertTrue(any("concurrency" in v for v in violations(bad)))

    def test_a_deploy_must_not_cancel_a_running_rollout(self):
        bad = mutate("cancel-in-progress: false", "cancel-in-progress: true")
        self.assertTrue(any("concurrency" in v for v in violations(bad)))

    def test_traffic_must_follow_the_check(self):
        steps = GOOD_DEPLOY.split("      - name: rollout")[0]
        bad = steps + (
            "      - name: traffic\n        run: gcloud run services update-traffic svc --to-latest\n"
            '      - name: rollout\n        run: scripts/rollout-check.sh "$URL" "$HEADER_FILE"\n'
        )
        self.assertTrue(any("traffic" in v for v in violations(bad)))

    def test_a_deploy_without_a_check_is_refused(self):
        bad = mutate('run: scripts/rollout-check.sh "$URL" "$HEADER_FILE"', "run: echo skipped")
        self.assertTrue(any("rollout-check" in v for v in violations(bad)))

    def test_a_build_secret_is_refused(self):
        bad = mutate("push: true", "push: true\n          secrets: gh_token=${{ steps.t.outputs.token }}")
        self.assertTrue(any("build-push" in v for v in violations(bad)))

    def test_an_ssh_forward_is_refused(self):
        bad = mutate("push: true", "push: true\n          ssh: default")
        self.assertTrue(any("build-push" in v for v in violations(bad)))

    def test_a_checkout_must_not_persist_credentials(self):
        bad = mutate("          persist-credentials: false\n", "")
        self.assertTrue(any("persist-credentials" in v for v in violations(bad)))

    def test_an_unversioned_action_is_refused(self):
        bad = mutate("actions/checkout@v7", "actions/checkout")
        self.assertTrue(any("version" in v for v in violations(bad)))

    def test_the_action_is_checked_too(self):
        action = """
runs:
  using: composite
  steps:
    - uses: actions/checkout@v7
    - shell: bash
      run: echo ${{ secrets.X }}
"""
        found = violations(actions={"siblings": action})
        self.assertTrue(any("persist-credentials" in v for v in found))
        self.assertTrue(any("secret in a run" in v for v in found))


class RolloutCheckCalls(unittest.TestCase):
    """deploy.yml calls scripts/rollout-check.sh by its real interface."""

    def test_the_build_gets_the_branch_and_the_sha(self):
        for name in ("BUILD_BRANCH", "BUILD_SHA"):
            bad = mutate(f"            {name}=" + ("${{ github.ref_name }}" if name == "BUILD_BRANCH" else "${{ github.sha }}") + "\n", "")
            self.assertTrue(any(name in v for v in violations(bad)), name)

    def test_the_build_args_come_from_the_ref_and_the_sha(self):
        bad = mutate("BUILD_SHA=${{ github.sha }}", "BUILD_SHA=abc")
        self.assertTrue(any("BUILD_SHA" in v for v in violations(bad)))

    def test_the_public_host_is_checked_through_the_worker(self):
        bad = mutate("--via-worker", "")
        self.assertTrue(any("--via-worker" in v for v in violations(bad)))

    def test_the_bare_origin_is_checked_refused(self):
        bad = mutate("--refused", "")
        self.assertTrue(any("--refused" in v for v in violations(bad)))

    def test_the_worker_check_never_gets_the_secret(self):
        bad = mutate('--via-worker "https://$HOST"', '--via-worker "https://$HOST" "$HEADER_FILE"')
        self.assertTrue(any("--via-worker" in v and "secret" in v for v in violations(bad)))

    def test_the_refused_check_never_gets_the_secret(self):
        bad = mutate('--refused "$SERVICE_URL" ||', 'ORIGIN_SECRET=x scripts/rollout-check.sh --refused "$SERVICE_URL" ||')
        self.assertTrue(any("--refused" in v and "secret" in v for v in violations(bad)))

    def test_a_challenge_is_a_warning_not_a_failure(self):
        bad = mutate('::warning::challenged', 'challenged')
        self.assertTrue(any("exit 3" in v for v in violations(bad)))

    def test_no_inline_curl_in_the_deploy(self):
        bad = mutate('scripts/rollout-check.sh --refused "$SERVICE_URL" ||', 'curl "$SERVICE_URL" ||')
        found = violations(bad)
        self.assertTrue(any("curl" in v for v in found))


class ServingRevision(unittest.TestCase):
    """The skip and the rollback look at the revision that serves (D7, D11)."""

    SERVING = '$(gcloud run services describe svc --format json | scripts/serving-revision.sh) || { echo "::error::no revision serves svc"; exit 1; }'

    def test_the_skip_compares_the_revision_that_serves_not_the_spec(self):
        bad = mutate(self.SERVING, "$(gcloud run services describe svc --format='value(spec.template.spec.containers[0].image)')")
        self.assertTrue(any("serving revision" in v for v in violations(bad)), violations(bad))

    def test_the_serving_revision_is_read_by_the_script_that_knows_latest_revision(self):
        # The inline jq of before read `.revisionName` only: empty for a
        # service that follows the latest revision (what tofu creates).
        inline = "$(gcloud run services describe svc --format json | jq -r '[.status.traffic[] | select(.percent == 100) | .revisionName] | .[0] // empty')"
        bad = mutate(self.SERVING, inline)
        self.assertTrue(any("serving-revision.sh" in v for v in violations(bad)), violations(bad))

    def test_an_empty_serving_revision_fails_the_job_loudly(self):
        bad = mutate(self.SERVING, "$(gcloud run services describe svc --format json | scripts/serving-revision.sh)")
        self.assertTrue(any("::error::" in v and "serving" in v for v in violations(bad)), violations(bad))

    def test_the_pipe_to_the_script_is_not_masked(self):
        bad = mutate("set -euo pipefail\n          REV=", "set -eu\n          REV=")
        self.assertTrue(any("pipefail" in v for v in violations(bad)), violations(bad))

    def test_the_skip_reads_the_image_of_that_revision(self):
        bad = mutate("gcloud run revisions describe", "gcloud run services describe")
        self.assertTrue(any("serving revision" in v for v in violations(bad)), violations(bad))

    def test_the_serving_revision_is_recorded_before_the_traffic_moves(self):
        bad = mutate('          echo "serving=$REV" >> "$GITHUB_OUTPUT"\n', "")
        self.assertTrue(any("serving=" in v for v in violations(bad)), violations(bad))

    def test_a_failed_public_check_moves_the_traffic_back(self):
        bad = mutate('gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100" || echo "::error::rollback to $PREVIOUS failed"', "true")
        self.assertTrue(any("roll" in v and "back" in v for v in violations(bad)), violations(bad))

    def test_the_rollback_goes_to_the_recorded_revision(self):
        bad = mutate("PREVIOUS: ${{ steps.current.outputs.serving }}", "PREVIOUS: ${{ github.sha }}")
        self.assertTrue(any("roll" in v and "back" in v for v in violations(bad)), violations(bad))

    def test_a_cloudflare_challenge_does_not_move_the_traffic_back(self):
        bad = mutate('echo "::warning::challenged"',
                     'echo "::warning::challenged"; gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100"')
        self.assertTrue(any("challenge" in v and "back" in v for v in violations(bad)), violations(bad))


class RollbackFailure(unittest.TestCase):
    """A rollback that fails must not hide the check's exit code (set -e)."""

    GUARD = ' || echo "::error::rollback to $PREVIOUS failed"'

    def test_a_rollback_that_can_abort_the_step_is_refused(self):
        bad = mutate(self.GUARD, "")
        self.assertTrue(any("mask" in v for v in violations(bad)), violations(bad))

    def test_the_failed_rollback_is_reported_as_an_error(self):
        bad = mutate(self.GUARD, " || true")
        self.assertTrue(any("mask" in v for v in violations(bad)), violations(bad))

    def test_the_step_still_ends_with_the_checks_exit_code(self):
        bad = mutate('            exit "$rc"\n          fi\n          rc=0\n',
                     "          fi\n          rc=0\n")
        self.assertTrue(any("exit" in v and "code" in v for v in violations(bad)), violations(bad))


class WorkerFiveHundred(unittest.TestCase):
    """The Worker's own 500 (exit 4) names its remedy and rolls nothing back."""

    LINE = 'echo "::error::the Worker answered 500 — run infra/scripts/origin-secret push $ENV"'
    ROLLBACK = 'gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100" || echo "::error::rollback to $PREVIOUS failed"'

    def test_the_arm_for_exit_4_must_exist(self):
        bad = mutate('          elif [ "$rc" = 4 ]; then\n            ' + self.LINE + '\n            exit "$rc"\n', "")
        self.assertTrue(any("exit 4" in v for v in violations(bad)), violations(bad))

    def test_the_arm_for_exit_4_names_the_remedy(self):
        bad = mutate(self.LINE, 'echo "::error::the Worker answered 500"')
        self.assertTrue(any("origin-secret push" in v for v in violations(bad)), violations(bad))

    def test_the_arm_for_exit_4_is_an_error_annotation(self):
        bad = mutate(self.LINE, 'echo "the Worker answered 500 — run infra/scripts/origin-secret push $ENV"')
        self.assertTrue(any("origin-secret push" in v for v in violations(bad)), violations(bad))

    def test_a_worker_500_does_not_move_the_traffic_back(self):
        bad = mutate(self.LINE, self.LINE + "; " + self.ROLLBACK)
        found = violations(bad)
        self.assertTrue(any("Worker" in v and "back" in v for v in found), found)

    def test_a_worker_500_still_ends_with_the_exit_code(self):
        bad = mutate('echo "::error::the Worker answered 500 — run infra/scripts/origin-secret push $ENV"\n            exit "$rc"\n',
                     'echo "::error::the Worker answered 500 — run infra/scripts/origin-secret push $ENV"\n')
        self.assertTrue(any("exit 4" in v for v in violations(bad)), violations(bad))


class RefusedOrigin(unittest.TestCase):
    """A bare *.run.app that answers fails the job and moves NO traffic (D13, round 3).

    Both revisions boot from the same env: a rollback restores nothing, so the
    job fails naming the missing `ORIGIN_SECRET` env of the service.
    """

    CALL = 'scripts/rollout-check.sh --refused "$SERVICE_URL" || rc=$?'
    ERROR = (
        'echo "::error::the bare origin answers — the service\'s ORIGIN_SECRET env is missing '
        'or empty (infra: the secret reference); traffic left as is"'
    )
    BLOCK = (
        '          rc=0\n'
        f'          {CALL}\n'
        '          if [ "$rc" != 0 ]; then\n'
        f'            {ERROR}\n'
        '            exit "$rc"\n'
        '          fi\n'
    )

    def test_the_block_is_in_the_good_fixture(self):
        self.assertIn(self.BLOCK, GOOD_DEPLOY)

    def test_an_origin_that_answers_moves_no_traffic_back(self):
        bad = mutate(self.BLOCK, self.BLOCK.replace(self.ERROR, self.ERROR + "\n            rollback"))
        found = violations(bad)
        self.assertTrue(any("refused" in v and "no rollback" in v for v in found), found)

    def test_a_direct_rollback_in_the_refused_arm_is_caught_too(self):
        direct = 'gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100" || echo "::error::rollback to $PREVIOUS failed"'
        bad = mutate(self.BLOCK, self.BLOCK.replace(self.ERROR, self.ERROR + "\n            " + direct))
        found = violations(bad)
        self.assertTrue(any("refused" in v and "no rollback" in v for v in found), found)

    def test_a_rollback_on_exit_1_of_the_refused_check_is_caught(self):
        bad = mutate(self.BLOCK, self.BLOCK.replace(self.ERROR, 'if [ "$rc" = 1 ]; then rollback; fi'))
        found = violations(bad)
        self.assertTrue(any("refused" in v and "no rollback" in v for v in found), found)

    def test_a_bare_refused_call_does_not_fail_the_job(self):
        bad = mutate(self.BLOCK, '          scripts/rollout-check.sh --refused "$SERVICE_URL"\n')
        found = violations(bad)
        self.assertTrue(any("refused" in v and "::error::" in v for v in found), found)

    def test_the_error_names_the_missing_origin_secret_env(self):
        bad = mutate(self.ERROR, 'echo "::error::the bare origin answers"')
        found = violations(bad)
        self.assertTrue(any("refused" in v and "ORIGIN_SECRET" in v for v in found), found)

    def test_the_error_is_an_annotation(self):
        bad = mutate(self.ERROR, self.ERROR.replace("::error::", ""))
        found = violations(bad)
        self.assertTrue(any("refused" in v and "::error::" in v for v in found), found)

    def test_the_refused_arm_still_ends_with_the_exit_code(self):
        bad = mutate(self.BLOCK, self.BLOCK.replace('            exit "$rc"\n', ""))
        found = violations(bad)
        self.assertTrue(any("refused" in v and "exit" in v for v in found), found)


class RollbackFunction(unittest.TestCase):
    """`rollback` is one function called from several arms: its arms are the callers'."""

    CALL = 'if [ "$rc" = 1 ]; then rollback; fi'

    def test_a_function_nobody_calls_rolls_nothing_back(self):
        bad = GOOD_DEPLOY.replace(self.CALL, "true")
        self.assertNotIn(self.CALL, bad)
        found = violations(bad)
        self.assertTrue(any("roll" in v and "back" in v for v in found), found)

    def test_a_call_in_the_challenge_arm_is_caught(self):
        bad = mutate('echo "::warning::challenged"', 'echo "::warning::challenged"; rollback')
        found = violations(bad)
        self.assertTrue(any("challenge" in v and "back" in v for v in found), found)

    def test_a_call_in_the_worker_500_arm_is_caught(self):
        bad = mutate(WorkerFiveHundred.LINE, WorkerFiveHundred.LINE + "; rollback")
        found = violations(bad)
        self.assertTrue(any("Worker" in v and "back" in v for v in found), found)

    def test_a_call_outside_every_arm_is_caught(self):
        bad = mutate(self.CALL, "true").replace("rc=0\n          scripts/rollout-check.sh --via-worker", "rollback\n          rc=0\n          scripts/rollout-check.sh --via-worker", 1)
        found = violations(bad)
        self.assertTrue(any("refusal" in v for v in found), found)


class RollbackArm(unittest.TestCase):
    """The rollback sits in the arm whose condition is the refusal exit code."""

    ARMS = (
        '          if [ "$rc" = 3 ]; then\n'
        '            echo "::warning::challenged"\n'
        '          elif [ "$rc" = 4 ]; then\n'
        '            echo "::error::the Worker answered 500 — run infra/scripts/origin-secret push $ENV"\n'
        '            exit "$rc"\n'
        '          elif [ "$rc" != 0 ]; then\n'
        '            if [ "$rc" = 1 ]; then rollback; fi\n'
        '            exit "$rc"\n'
        '          fi\n'
    )
    ROLLBACK = 'gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100" || echo "::error::rollback to $PREVIOUS failed"'

    def back(self, found):
        return [v for v in found if "roll" in v and "back" in v]

    def test_the_arms_of_the_good_fixture_are_what_these_tests_swap(self):
        self.assertIn(self.ARMS, GOOD_DEPLOY)

    def test_swapping_the_bodies_of_the_arms_is_caught(self):
        swapped = (
            '          if [ "$rc" = 3 ]; then\n'
            f"            {self.ROLLBACK}\n"
            '          elif [ "$rc" != 0 ]; then\n'
            '            echo "::warning::challenged"\n'
            '            exit "$rc"\n'
            "          fi\n"
        )
        found = self.back(violations(mutate(self.ARMS, swapped)))
        self.assertTrue(any("challenge" in v for v in found), found)

    def test_a_rollback_on_any_failure_not_only_a_refusal_is_caught(self):
        bare = self.ARMS.replace(
            'if [ "$rc" = 1 ]; then rollback; fi', self.ROLLBACK
        )
        self.assertNotEqual(bare, self.ARMS)
        found = self.back(violations(mutate(self.ARMS, bare)))
        self.assertTrue(any("refusal" in v for v in found), found)

    def test_a_rollback_outside_every_arm_is_caught(self):
        outside = (
            self.ARMS.replace(
                'if [ "$rc" = 1 ]; then rollback; fi', "true"
            )
            + f"          {self.ROLLBACK}\n"
        )
        found = self.back(violations(mutate(self.ARMS, outside)))
        self.assertTrue(any("refusal" in v for v in found), found)

    def test_the_arm_is_found_whatever_the_layout(self):
        # The refusal block on several lines, the rollback not on the line of
        # its `if`: still inside the arm, so still clean.
        spread = self.ARMS.replace(
            'if [ "$rc" = 1 ]; then rollback; fi',
            f'if [ "$rc" = 1 ]; then\n              {self.ROLLBACK}\n            fi',
        )
        self.assertNotEqual(spread, self.ARMS)
        # No call of the function is left: its body is the one in the arm.
        inline = re.sub(
            r"rollback\(\) \{.*?\n          \}", "rollback() {\n            true\n          }",
            mutate(self.ARMS, spread), count=1, flags=re.S,
        )
        self.assertEqual(violations(inline), [])


class RollbackOutcome(unittest.TestCase):
    """The rollback is observed: the previous revision serves 100%, else exit 5 (round 3)."""

    READ = '            NOW=$(gcloud run services describe svc --format json | scripts/serving-revision.sh) || NOW=""\n'
    BRANCH = (
        '            if [ "$NOW" != "$PREVIOUS" ]; then\n'
        '              echo "::error::rollback did not restore $PREVIOUS"\n'
        '              exit 5\n'
        '            fi\n'
    )

    def test_the_outcome_is_in_the_good_fixture(self):
        self.assertIn(self.READ + self.BRANCH, GOOD_DEPLOY)

    def test_a_rollback_that_is_not_observed_is_caught(self):
        found = violations(mutate(self.READ + self.BRANCH, ""))
        self.assertTrue(any("rollback" in v and "exit 5" in v for v in found), found)

    def test_the_traffic_is_read_after_the_rollback(self):
        found = violations(mutate(self.READ, ""))
        self.assertTrue(any("rollback" in v and "status.traffic" in v for v in found), found)

    def test_the_traffic_is_read_by_the_serving_revision_script(self):
        bad = mutate(self.READ, '            NOW=$(gcloud run services describe svc --format="value(status.url)") || NOW=""\n')
        found = violations(bad)
        self.assertTrue(any("rollback" in v and "status.traffic" in v for v in found), found)

    def test_a_read_before_the_rollback_does_not_count(self):
        update = '            gcloud run services update-traffic svc --to-revisions="$PREVIOUS=100" || echo "::error::rollback to $PREVIOUS failed"\n'
        bad = mutate(update + self.READ, self.READ + update)
        found = violations(bad)
        self.assertTrue(any("rollback" in v and "status.traffic" in v for v in found), found)

    def test_a_failed_rollback_ends_with_exit_5(self):
        found = violations(mutate("              exit 5\n", "              exit 1\n"))
        self.assertTrue(any("rollback" in v and "exit 5" in v for v in found), found)

    def test_the_failure_is_an_error_annotation_naming_the_revision(self):
        bad = mutate('echo "::error::rollback did not restore $PREVIOUS"', 'echo "rollback did not restore $PREVIOUS"')
        found = violations(bad)
        self.assertTrue(any("rollback" in v and "did not restore" in v for v in found), found)

    def test_exit_5_outside_the_rollback_function_is_caught(self):
        outside = mutate(self.BRANCH, "").replace(
            'echo "::warning::challenged"', 'echo "::warning::challenged"; exit 5', 1
        )
        found = violations(outside)
        self.assertTrue(any("rollback" in v and "exit 5" in v for v in found), found)

    def test_the_checks_exit_code_still_follows_the_rollback(self):
        # The success path leaves the function and the arm goes on to its exit.
        bad = mutate('            if [ "$rc" = 1 ]; then rollback; fi\n            exit "$rc"\n          fi\n          rc=0\n',
                     '            if [ "$rc" = 1 ]; then rollback; fi\n          fi\n          rc=0\n')
        found = violations(bad)
        self.assertTrue(any("exit" in v and "code" in v for v in found), found)


class Loading(unittest.TestCase):
    def test_a_file_that_does_not_parse_is_reported_not_raised(self):
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            (root / ".github" / "workflows").mkdir(parents=True)
            (root / ".github" / "workflows" / "ci.yml").write_text("name: a: b: c\n")
            found = cw.check_repository(root)
        self.assertTrue(any("ci.yml" in v and "YAML" in v for v in found), found)

    def test_no_workflow_at_all_is_reported(self):
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            self.assertTrue(cw.check_repository(pathlib.Path(tmp)))


class RealFiles(unittest.TestCase):
    def test_the_repository_workflows_pass(self):
        found = cw.check_repository(pathlib.Path(__file__).resolve().parent.parent)
        self.assertEqual(found, [])


if __name__ == "__main__":
    unittest.main()
