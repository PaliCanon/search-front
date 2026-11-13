"""Tests for scripts/check_workflows.py (task 007, slice S5; GHCR deploy).

Each rule has a workflow that breaks it (the checker names the rule) and the
real workflows pass. Run: python3 scripts/check_workflows_test.py
"""

import pathlib
import sys
import unittest

import yaml

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import check_workflows as cw  # noqa: E402

GOOD_DEPLOY = """
name: deploy
on:
  push:
    tags: ["v*"]
concurrency:
  group: deploy-prod
  cancel-in-progress: false
permissions:
  contents: read
env:
  IMAGE: ghcr.io/palicanon/search-front
jobs:
  image:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v7
        with:
          path: search-front
          persist-credentials: false
      - uses: ./search-front/.github/actions/siblings
        with:
          app-id: ${{ secrets.APP_ID }}
          private-key: ${{ secrets.APP_PRIVATE_KEY }}
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: vars
        id: vars
        run: |
          set -euo pipefail
          TAG=$(scripts/image-hash.sh ../ui-lib ../api)
          echo "tag=$TAG" >> "$GITHUB_OUTPUT"
          echo "image_hash=$IMAGE:$TAG" >> "$GITHUB_OUTPUT"
      - name: check
        id: check-image
        env:
          IMAGE_HASH: ${{ steps.vars.outputs.image_hash }}
        run: |
          set -euo pipefail
          if docker buildx imagetools inspect "$IMAGE_HASH" >/dev/null 2>&1; then
            echo "exists=true" >> "$GITHUB_OUTPUT"
          else
            echo "exists=false" >> "$GITHUB_OUTPUT"
          fi
      - uses: docker/build-push-action@v6
        if: steps.check-image.outputs.exists != 'true'
        with:
          context: .
          file: search-front/Dockerfile
          push: true
          labels: |
            org.opencontainers.image.source=https://github.com/${{ github.repository }}
          tags: |
            ${{ steps.vars.outputs.image_hash }}
            ${{ env.IMAGE }}:${{ github.ref_name }}
      - name: tag the existing image
        if: steps.check-image.outputs.exists == 'true'
        env:
          IMAGE_HASH: ${{ steps.vars.outputs.image_hash }}
          RELEASE: ${{ github.ref_name }}
        run: docker buildx imagetools create --tag "$IMAGE:$RELEASE" "$IMAGE_HASH"
      - name: summary
        env:
          IMAGE_HASH: ${{ steps.vars.outputs.image_hash }}
          TAG: ${{ steps.vars.outputs.tag }}
          RELEASE: ${{ github.ref_name }}
        run: |
          set -euo pipefail
          DIGEST=$(docker buildx imagetools inspect "$IMAGE_HASH" --format '{{.Manifest.Digest}}')
          test -n "$DIGEST" || { echo "::error::no digest for $IMAGE_HASH"; exit 1; }
          {
            echo "### search-front $RELEASE"
            echo "- image: $IMAGE@$DIGEST"
          } >> "$GITHUB_STEP_SUMMARY"
          echo "search-front: $IMAGE@$DIGEST ($TAG, release $RELEASE)"
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


def has(found, *words):
    return any(all(w in v for w in words) for v in found)


class Rules(unittest.TestCase):
    def test_a_good_deploy_passes(self):
        self.assertEqual(violations(), [])

    def test_a_secret_in_a_run_line_is_refused(self):
        bad = mutate('echo "tag=$TAG" >> "$GITHUB_OUTPUT"', 'echo "${{ secrets.TOKEN }}"')
        self.assertTrue(has(violations(bad), "secret in a run"))

    def test_a_secret_in_a_with_is_allowed(self):
        ok = mutate("push: true", "push: true\n          token: ${{ secrets.TOKEN }}")
        self.assertEqual(violations(ok), [])

    def test_the_concurrency_group_is_the_prod_one(self):
        bad = mutate("group: deploy-prod", "group: deploy")
        self.assertTrue(has(violations(bad), "concurrency"))

    def test_a_dev_concurrency_group_is_refused(self):
        bad = mutate("group: deploy-prod",
                     "group: deploy-${{ startsWith(github.ref, 'refs/tags/') && 'prod' || 'dev' }}")
        self.assertTrue(has(violations(bad), "concurrency"))

    def test_a_push_to_a_branch_deploys_nothing(self):
        bad = mutate('    tags: ["v*"]', '    branches: [main]\n    tags: ["v*"]')
        self.assertTrue(has(violations(bad), "branches"))

    def test_only_a_v_tag_triggers_the_deploy(self):
        bad = mutate('    tags: ["v*"]', '    tags: ["*"]')
        self.assertTrue(has(violations(bad), "v*"))

    def test_no_dev_environment_in_the_deploy(self):
        bad = mutate("jobs:", "  ENV: ${{ startsWith(github.ref, 'refs/tags/') && 'prod' || 'dev' }}\njobs:")
        self.assertTrue(has(violations(bad), "dev"))

    def test_a_deploy_must_not_cancel_a_running_run(self):
        bad = mutate("cancel-in-progress: false", "cancel-in-progress: true")
        self.assertTrue(has(violations(bad), "concurrency"))

    def test_a_build_secret_is_refused(self):
        bad = mutate("push: true", "push: true\n          secrets: gh_token=${{ steps.t.outputs.token }}")
        self.assertTrue(has(violations(bad), "build-push"))

    def test_an_ssh_forward_is_refused(self):
        bad = mutate("push: true", "push: true\n          ssh: default")
        self.assertTrue(has(violations(bad), "build-push"))

    def test_a_checkout_must_not_persist_credentials(self):
        bad = mutate("          persist-credentials: false\n", "")
        self.assertTrue(has(violations(bad), "persist-credentials"))

    def test_an_unversioned_action_is_refused(self):
        bad = mutate("actions/checkout@v7", "actions/checkout")
        self.assertTrue(has(violations(bad), "version"))

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
        self.assertTrue(has(found, "persist-credentials"))
        self.assertTrue(has(found, "secret in a run"))


class Permissions(unittest.TestCase):
    """Workflow level: read only. Job level: read, none, or `packages: write`."""

    WORKFLOW = "permissions:\n  contents: read\n"

    def test_missing_workflow_permissions_are_refused(self):
        bad = mutate(self.WORKFLOW, "")
        self.assertTrue(has(violations(bad), "permissions"))

    def test_a_workflow_write_permission_is_refused(self):
        bad = mutate(self.WORKFLOW, "permissions:\n  contents: write\n")
        self.assertTrue(has(violations(bad), "permissions", "contents"))

    def test_packages_write_is_refused_at_workflow_level(self):
        bad = mutate(self.WORKFLOW, "permissions:\n  contents: read\n  packages: write\n")
        self.assertTrue(has(violations(bad), "permissions", "packages"))

    def test_packages_write_is_allowed_at_job_level(self):
        self.assertEqual([v for v in violations() if "permissions" in v], [])

    def test_a_job_write_permission_other_than_packages_is_refused(self):
        bad = mutate("      contents: read\n      packages: write", "      contents: write\n      packages: write")
        self.assertTrue(has(violations(bad), "permissions", "contents"))
        bad = mutate("      packages: write", "      packages: write\n      pull-requests: write")
        self.assertTrue(has(violations(bad), "permissions", "pull-requests"))

    def test_id_token_write_is_refused_at_workflow_level(self):
        bad = mutate(self.WORKFLOW, "permissions:\n  contents: read\n  id-token: write\n")
        self.assertTrue(has(violations(bad), "id-token"))

    def test_id_token_write_is_refused_at_job_level(self):
        bad = mutate("      packages: write", "      packages: write\n      id-token: write")
        self.assertTrue(has(violations(bad), "id-token"))


class GhcrLogin(unittest.TestCase):
    """The image job logs in to GHCR with its own token, and may write packages."""

    LOGIN = (
        "      - uses: docker/login-action@v3\n"
        "        with:\n"
        "          registry: ghcr.io\n"
        "          username: ${{ github.actor }}\n"
        "          password: ${{ secrets.GITHUB_TOKEN }}\n"
    )

    def test_the_login_is_in_the_good_fixture(self):
        self.assertIn(self.LOGIN, GOOD_DEPLOY)

    def test_a_job_that_pushes_without_logging_in_is_refused(self):
        self.assertTrue(has(violations(mutate(self.LOGIN, "")), "docker/login-action"))

    def test_the_registry_is_ghcr(self):
        bad = mutate("registry: ghcr.io", "registry: europe-docker.pkg.dev")
        self.assertTrue(has(violations(bad), "login", "registry"))

    def test_the_username_is_the_actor(self):
        bad = mutate("username: ${{ github.actor }}", "username: someone")
        self.assertTrue(has(violations(bad), "login", "username"))

    def test_the_password_is_the_job_token(self):
        bad = mutate("password: ${{ secrets.GITHUB_TOKEN }}", "password: ${{ secrets.GHCR_PAT }}")
        self.assertTrue(has(violations(bad), "login", "password"))

    def test_the_password_is_given_by_with_only(self):
        bad = mutate(self.LOGIN, self.LOGIN.replace("          password: ${{ secrets.GITHUB_TOKEN }}\n", "")
                     + "      - run: echo ${{ secrets.GITHUB_TOKEN }} | docker login ghcr.io -u x --password-stdin\n")
        found = violations(bad)
        self.assertTrue(has(found, "login", "password"))
        self.assertTrue(has(found, "secret in a run"))

    def test_the_job_with_the_login_has_job_permissions(self):
        bad = mutate("    permissions:\n      contents: read\n      packages: write\n", "")
        self.assertTrue(has(violations(bad), "login", "permissions"))

    def test_the_job_with_the_login_may_write_packages(self):
        bad = mutate("      packages: write", "      packages: read")
        self.assertTrue(has(violations(bad), "packages: write"))

    def test_the_job_with_the_login_reads_the_contents(self):
        bad = mutate("      contents: read\n      packages: write", "      packages: write")
        self.assertTrue(has(violations(bad), "contents: read"))


class NoGcpLeft(unittest.TestCase):
    """Nothing of Cloud Run, gcloud or the Worker is left in the deploy."""

    def test_a_google_action_is_refused(self):
        bad = mutate("      - uses: docker/setup-buildx-action@v3\n",
                     "      - uses: docker/setup-buildx-action@v3\n      - uses: google-github-actions/auth@v2\n")
        self.assertTrue(has(violations(bad), "google-github-actions"))

    def test_gcloud_in_a_run_line_is_refused(self):
        bad = mutate('echo "tag=$TAG" >> "$GITHUB_OUTPUT"', "gcloud run services list")
        self.assertTrue(has(violations(bad), "gcloud"))

    def test_run_app_in_a_run_line_is_refused(self):
        bad = mutate('echo "tag=$TAG" >> "$GITHUB_OUTPUT"', "echo https://svc-x.a.run.app")
        self.assertTrue(has(violations(bad), "run.app"))

    def test_an_artifact_registry_in_a_run_line_is_refused(self):
        bad = mutate('echo "tag=$TAG" >> "$GITHUB_OUTPUT"', "echo europe-docker.pkg.dev/p/r/i")
        self.assertTrue(has(violations(bad), "docker.pkg.dev"))

    def test_update_traffic_in_a_run_line_is_refused(self):
        bad = mutate('echo "tag=$TAG" >> "$GITHUB_OUTPUT"', "echo update-traffic")
        self.assertTrue(has(violations(bad), "update-traffic"))

    def test_via_worker_in_a_run_line_is_refused(self):
        bad = mutate('echo "tag=$TAG" >> "$GITHUB_OUTPUT"', "scripts/rollout-check.sh --via-worker x")
        self.assertTrue(has(violations(bad), "--via-worker"))

    def test_an_id_token_permission_of_any_level_is_refused(self):
        bad = mutate("      packages: write", "      packages: write\n      id-token: none")
        self.assertTrue(has(violations(bad), "id-token"))

    def test_a_comment_naming_gcloud_is_not_a_run_line(self):
        ok = mutate("          set -euo pipefail\n          TAG=", "          set -euo pipefail\n          # no gcloud here\n          TAG=")
        self.assertEqual(violations(ok), [])


class ImageBuild(unittest.TestCase):
    """The build-push step pushes, labels the source and tags the release."""

    def test_the_build_pushes(self):
        self.assertTrue(has(violations(mutate("push: true", "push: false")), "push"))
        self.assertTrue(has(violations(mutate("          push: true\n", "")), "push"))

    def test_the_image_is_labelled_with_its_source(self):
        bad = mutate("          labels: |\n            org.opencontainers.image.source=https://github.com/${{ github.repository }}\n", "")
        self.assertTrue(has(violations(bad), "labels"))
        bad = mutate("source=https://github.com/${{ github.repository }}", "source=https://github.com/other/repo")
        self.assertTrue(has(violations(bad), "labels"))

    def test_the_tags_name_ghcr(self):
        bad = mutate("IMAGE: ghcr.io/palicanon/search-front", "IMAGE: europe-docker.pkg.dev/p/r/search-front")
        self.assertTrue(has(violations(bad), "tags", "ghcr.io"))

    def test_the_tags_name_ghcr_by_a_literal_too(self):
        ok = mutate("${{ env.IMAGE }}:${{ github.ref_name }}", "ghcr.io/palicanon/search-front:${{ github.ref_name }}")
        self.assertEqual(violations(ok), [])

    def test_the_tags_carry_the_release_tag(self):
        bad = mutate("            ${{ env.IMAGE }}:${{ github.ref_name }}\n", "")
        self.assertTrue(has(violations(bad), "tags", "github.ref_name"))

    def test_the_image_build_takes_no_build_arg(self):
        for arg in ("BUILD_SHA=${{ github.sha }}", "GIT_SHA=${{ github.sha }}", "RUST_VERSION=1.91"):
            bad = mutate("push: true", "push: true\n          build-args: |\n            " + arg)
            self.assertTrue(has(violations(bad), "build-args"), arg)


class ExistenceCheck(unittest.TestCase):
    """The image is built only when it does not exist, and the check asks the registry."""

    INSPECT = 'if docker buildx imagetools inspect "$IMAGE_HASH" >/dev/null 2>&1; then'

    def test_the_check_uses_imagetools_inspect(self):
        bad = mutate(self.INSPECT, 'if docker manifest inspect "$IMAGE_HASH" >/dev/null 2>&1; then')
        self.assertTrue(has(violations(bad), "imagetools inspect"))

    def test_the_build_is_skipped_when_the_image_exists(self):
        bad = mutate("        if: steps.check-image.outputs.exists != 'true'\n", "")
        self.assertTrue(has(violations(bad), "exists != 'true'"))

    def test_the_skip_reads_the_output_of_a_checking_step(self):
        bad = mutate("if: steps.check-image.outputs.exists != 'true'", "if: steps.vars.outputs.exists != 'true'")
        self.assertTrue(has(violations(bad), "imagetools inspect"))

    def test_the_skip_reads_a_step_that_exists(self):
        bad = mutate("if: steps.check-image.outputs.exists != 'true'", "if: steps.nothing.outputs.exists != 'true'")
        self.assertTrue(has(violations(bad), "imagetools inspect"))


class DigestSummary(unittest.TestCase):
    """The run's summary gives the digest the stack pulls by."""

    def test_a_step_writes_the_summary(self):
        bad = mutate('          } >> "$GITHUB_STEP_SUMMARY"\n', "          } >> /dev/null\n")
        self.assertTrue(has(violations(bad), "GITHUB_STEP_SUMMARY"))

    def test_the_summary_gives_the_image_by_digest(self):
        bad = mutate("$IMAGE@$DIGEST", "$IMAGE:$TAG")
        self.assertTrue(has(violations(bad), "@$DIGEST"))

    def test_the_digest_is_read_from_the_registry(self):
        bad = mutate("DIGEST=$(docker buildx imagetools inspect \"$IMAGE_HASH\" --format '{{.Manifest.Digest}}')",
                     "DIGEST=$(cat digest.txt)")
        self.assertTrue(has(violations(bad), "imagetools inspect", "summary"))


class NodeTestArguments(unittest.TestCase):
    """Task 007 F3: `node --test infra/worker/` passed on node 26 and failed
    on the runner's node 22 ("Cannot find module .../infra/worker"): node 22
    runs a directory argument as a module. The arguments are quoted globs."""

    def found(self, command):
        ci = "name: ci\non: push\npermissions:\n  contents: read\njobs:\n  j:\n    runs-on: x\n    steps:\n      - run: " + command + "\n"
        return violations(extra={"ci.yml": ci})

    def test_a_directory_argument_is_refused(self):
        for command in ("node --test infra/worker/", "node --test infra/worker", "node --test ."):
            found = self.found(command)
            self.assertTrue(any("node --test" in v for v in found), (command, found))

    def test_a_quoted_glob_passes(self):
        self.assertEqual(self.found("node --test 'infra/worker/*.test.mjs'"), [])
        self.assertEqual(self.found('node --test "infra/worker/*.test.mjs"'), [])

    def test_a_file_passes(self):
        self.assertEqual(self.found("node --test infra/worker/proxy.test.mjs"), [])


class SiblingsForCargo(unittest.TestCase):
    """Task 007 F4: the fmt job ran `cargo fmt --all --check` without the
    sibling checkouts; rustfmt follows `app/build.rs`'s `#[path]` into
    `../../ui-lib/build/placeholders.rs` and failed. Every job that runs
    cargo has the `siblings` action."""

    JOB = (
        "name: ci\non: push\npermissions:\n  contents: read\njobs:\n  j:\n    runs-on: x\n    steps:\n"
        "      - uses: actions/checkout@v7\n        with:\n          persist-credentials: false\n%s"
    )

    def found(self, steps):
        return violations(extra={"ci.yml": self.JOB % steps})

    def test_a_cargo_job_without_the_siblings_is_refused(self):
        found = self.found("      - run: cargo fmt --all --check\n")
        self.assertTrue(any("siblings" in v and "j" in v for v in found), found)

    def test_a_cargo_job_with_the_siblings_passes(self):
        steps = "      - uses: ./search-front/.github/actions/siblings\n      - run: cargo fmt --all --check\n"
        self.assertEqual(self.found(steps), [])

    def test_a_job_without_cargo_needs_no_siblings(self):
        self.assertEqual(self.found("      - run: node --test 'infra/worker/*.test.mjs'\n"), [])

    def test_the_fmt_job_of_ci_has_the_siblings(self):
        text = (pathlib.Path(__file__).resolve().parent.parent / ".github/workflows/ci.yml").read_text()
        fmt = parse(text)["jobs"]["fmt"]
        self.assertTrue(any(str(step.get("uses", "")).endswith("/siblings") for step in fmt["steps"]))


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
