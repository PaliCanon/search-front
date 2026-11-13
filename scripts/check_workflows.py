#!/usr/bin/env python3
"""Checks the rules the CI design names, on the workflows and composite actions.

No actionlint on the laptop and none in the job: these rules are the ones the
deploy design depends on (task 007), checked as YAML.

  python3 scripts/check_workflows.py

Rules:
- every file parses as YAML;
- a workflow declares `permissions`, read-only; a job may add `packages: write`
  (the one write scope, for the GHCR push) and never `id-token: write`;
- no `${{ secrets.* }}` inside a `run:` line (a secret goes through `env:` or
  `with:`, never into a shell command that a log or `set -x` could show);
- every `uses:` names a version (`@...`), local actions apart;
- every `actions/checkout` sets `persist-credentials: false` (the GitHub App
  token must not stay in `.git/config`);
- every job that runs `cargo` has the `siblings` action, `cargo fmt` included
  (rustfmt follows `app/build.rs`'s `#[path]` into ui-lib, F4 of 007);
- `node --test` takes files or quoted globs, never a directory (node 22 runs
  a directory as a module and fails with "Cannot find module", F3 of 007);
- no `docker/build-push-action` takes `secrets:` or `ssh:` (the build needs
  no credential, ruling R1);
- deploy.yml: a `v*` tag only, concurrency group `deploy-prod` that never
  cancels, no dev environment; the image job logs in to GHCR
  (`docker/login-action`, `ghcr.io`, `github.actor`, `secrets.GITHUB_TOKEN`
  by `with:`) and has job `permissions` `contents: read` + `packages: write`;
  no Cloud Run left (no google-github-actions, no gcloud, run.app,
  docker.pkg.dev, update-traffic, --via-worker, id-token); the build pushes,
  labels `org.opencontainers.image.source`, tags `ghcr.io/` and the release
  (`github.ref_name`), takes no `build-args`, and is skipped by
  `steps.<check>.outputs.exists != 'true'`, the check being
  `docker buildx imagetools inspect`; a step writes `$GITHUB_STEP_SUMMARY`
  with `imagetools inspect` and `@$DIGEST`.
Exit 0 clean, 1 with one line per violation.
"""

import pathlib
import re
import sys

import yaml


def steps_of(doc):
    """Every step of a workflow (all jobs) or of a composite action."""
    if "jobs" in doc:
        for job in doc["jobs"].values():
            yield from job.get("steps", [])
    else:
        yield from doc.get("runs", {}).get("steps", [])


def jobs_of(doc):
    return (doc.get("jobs") or {}).items()


def check_permissions(name, doc):
    """Workflow level: required, read or none. Job level: the same, plus `packages: write`."""
    if "jobs" not in doc:
        return []
    perms = doc.get("permissions")
    if not isinstance(perms, dict):
        return [f"{name}: permissions: a workflow must declare a minimal `permissions`"]
    found = [
        f"{name}: permissions: `{scope}: {level}` is more than read at workflow level"
        for scope, level in perms.items()
        if level not in ("read", "none")
    ]
    for job_name, job in jobs_of(doc):
        job_perms = job.get("permissions")
        if job_perms is None:
            continue
        if not isinstance(job_perms, dict):
            found.append(f"{name}: permissions: job `{job_name}` must list its scopes (read, or `packages: write`)")
            continue
        found += [
            f"{name}: permissions: job `{job_name}`: `{scope}: {level}` is more than read (only packages: write is allowed)"
            for scope, level in job_perms.items()
            if level not in ("read", "none") and not (scope == "packages" and level == "write")
        ]
    return found


def check_node_test(name, run):
    """`node --test DIR` runs the directory as a module on node 22 (task 007
    F3): the arguments must be files or quoted globs (node expands them)."""
    found = []
    for line in run.splitlines():
        match = re.search(r"\bnode\s+--test\s+(\S+)", line)
        if match and not (match.group(1)[0] in "'\"" or match.group(1).endswith((".mjs", ".js", ".cjs"))):
            found.append(f"{name}: node --test: `{match.group(1)}` is a directory or an unquoted glob; node 22 runs it as a module (use a quoted glob or a file)")
    return found


def check_step(name, step):
    found = []
    run = step.get("run", "")
    if "secrets." in run:
        found.append(f"{name}: secret in a run line: {run.strip().splitlines()[0][:60]}")
    found.extend(check_node_test(name, run))
    uses = step.get("uses", "")
    if uses and not uses.startswith("./") and "@" not in uses:
        found.append(f"{name}: `{uses}` names no version")
    with_ = step.get("with", {}) or {}
    if uses.startswith("actions/checkout@") or uses == "actions/checkout":
        if with_.get("persist-credentials") is not False:
            found.append(f"{name}: persist-credentials: an actions/checkout must set it to false")
    if uses.startswith("docker/build-push-action"):
        for key in ("secrets", "ssh"):
            if key in with_:
                found.append(f"{name}: build-push: `{key}:` is refused, the image build takes no credential")
    return found


def run_lines(doc):
    """Every non-comment line of every `run:` of the workflow, in order."""
    lines = []
    for step in steps_of(doc):
        for line in step.get("run", "").splitlines():
            if not line.strip().startswith("#"):
                lines.append(line)
    return lines



def check_siblings(name, doc):
    """Every job that runs cargo has the `siblings` action (task 007 F4):
    the workspace reaches `../ui-lib` and `../api`, and rustfmt follows
    `app/build.rs`'s `#[path]` into ui-lib, so even `cargo fmt` needs them."""
    found = []
    for job_name, job in (doc.get("jobs") or {}).items():
        steps = job.get("steps", [])
        runs_cargo = any(re.search(r"(^|[\s&;|(])cargo\s", step.get("run", ""), re.M) for step in steps)
        has_siblings = any(str(step.get("uses", "")).endswith("/siblings") for step in steps)
        if runs_cargo and not has_siblings:
            found.append(f"{name}: job `{job_name}` runs cargo without the siblings action (ui-lib and api beside the checkout)")
    return found



def build_steps(doc):
    """`(job_name, job, step)` of every docker/build-push-action step."""
    return [
        (job_name, job, step)
        for job_name, job in jobs_of(doc)
        for step in job.get("steps", [])
        if str(step.get("uses", "")).startswith("docker/build-push-action")
    ]


def check_login(name, job_name, job):
    """The job that pushes logs in to GHCR with its own token and may write packages."""
    login = next(
        (s for s in job.get("steps", []) if str(s.get("uses", "")).startswith("docker/login-action")),
        None,
    )
    if login is None:
        return [f"{name}: job `{job_name}` pushes without a docker/login-action step (login to ghcr.io)"]
    with_ = login.get("with") or {}
    found = []
    if with_.get("registry") != "ghcr.io":
        found.append(f"{name}: login: registry must be `ghcr.io`")
    if with_.get("username") != "${{ github.actor }}":
        found.append(f"{name}: login: username must be `${{{{ github.actor }}}}`")
    if with_.get("password") != "${{ secrets.GITHUB_TOKEN }}":
        found.append(f"{name}: login: password must be `${{{{ secrets.GITHUB_TOKEN }}}}`, given by `with:`")
    perms = job.get("permissions")
    if not isinstance(perms, dict):
        found.append(f"{name}: job `{job_name}` has the login but no job-level `permissions` (contents: read, packages: write)")
    else:
        for scope, level in (("packages", "write"), ("contents", "read")):
            if perms.get(scope) != level:
                found.append(f"{name}: job `{job_name}` permissions: `{scope}: {level}` is required by the GHCR push")
    return found


GCP_RUN = ("gcloud", "run.app", "docker.pkg.dev", "update-traffic", "--via-worker")


def check_no_gcp(name, doc):
    """Nothing of Cloud Run, gcloud, the Worker or the OIDC token is left."""
    found = []
    for step in steps_of(doc):
        if str(step.get("uses", "")).startswith("google-github-actions/"):
            found.append(f"{name}: no GCP here: `{step['uses']}` (google-github-actions) is refused")
    lines = run_lines(doc)
    for word in GCP_RUN:
        if any(word in line for line in lines):
            found.append(f"{name}: no GCP here: `{word}` in a run line")
    scopes = [doc.get("permissions")] + [job.get("permissions") for _, job in jobs_of(doc)]
    if any(isinstance(s, dict) and "id-token" in s for s in scopes):
        found.append(f"{name}: no GCP here: `id-token` is refused at any level (nothing needs the OIDC token)")
    return found


def expand_env(text, doc, job, step):
    """`${{ env.X }}` replaced by its value, from the step, the job, then the workflow."""
    env = {**(doc.get("env") or {}), **(job.get("env") or {}), **(step.get("env") or {})}
    return re.sub(r"\$\{\{\s*env\.(\w+)\s*\}\}", lambda m: str(env.get(m.group(1), m.group(0))), str(text))


SOURCE_LABEL = "org.opencontainers.image.source=https://github.com/${{ github.repository }}"


def check_build(name, doc, job_name, job, step):
    """The build pushes, labels its source, tags ghcr.io and the release, takes no build arg."""
    with_ = step.get("with") or {}
    found = []
    if with_.get("push") is not True:
        found.append(f"{name}: build-push: `push: true` is required (the image goes to GHCR)")
    if SOURCE_LABEL not in str(with_.get("labels", "")):
        found.append(f"{name}: build-push: labels must hold `{SOURCE_LABEL}` (links the package to the repository)")
    tags = expand_env(with_.get("tags", ""), doc, job, step)
    if "ghcr.io/" not in tags:
        found.append(f"{name}: build-push: tags must name `ghcr.io/` (via env IMAGE or a literal)")
    if "github.ref_name" not in tags:
        found.append(f"{name}: build-push: tags must carry the release tag (`github.ref_name`)")
    if "build-args" in with_:
        found.append(f"{name}: build-push: `build-args` is refused: the image does not change with the ref")
    found += check_exists_skip(name, job, step)
    return found


def check_exists_skip(name, job, step):
    """The build is skipped by `steps.<check>.outputs.exists != 'true'`; <check> asks the registry."""
    match = re.search(r"steps\.([\w-]+)\.outputs\.exists\s*!=\s*'true'", str(step.get("if", "")))
    if match is None:
        return [f"{name}: build-push: the build must be skipped by `if: steps.<check>.outputs.exists != 'true'`"]
    check = next((s for s in job.get("steps", []) if s.get("id") == match.group(1)), None)
    if check is None or "docker buildx imagetools inspect" not in check.get("run", ""):
        return [f"{name}: build-push: step `{match.group(1)}` must check existence with `docker buildx imagetools inspect`"]
    return []


def check_summary(name, doc):
    """A step writes the digest to the summary: what the stack pulls by."""
    steps = [s for s in steps_of(doc) if "GITHUB_STEP_SUMMARY" in s.get("run", "")]
    if not steps:
        return [f"{name}: summary: a step must write the digest to $GITHUB_STEP_SUMMARY"]
    if not any("@$DIGEST" in s["run"] for s in steps):
        return [f"{name}: summary: the summary must give the image by digest (`$IMAGE@$DIGEST`)"]
    if not any("imagetools inspect" in s["run"] for s in steps):
        return [f"{name}: summary: the digest must be read with `docker buildx imagetools inspect` in the summary step"]
    return []


def check_deploy(name, doc):
    found = []
    if doc.get("concurrency", {}).get("group") != "deploy-prod":
        found.append(f"{name}: concurrency: the group is the constant `deploy-prod` (prod is the only environment)")
    # PyYAML reads the key `on` as the boolean True.
    push = ((doc.get("on") or doc.get(True) or {}).get("push")) or {}
    if "branches" in push or "branches-ignore" in push:
        found.append(f"{name}: on.push.branches: a push to a branch deploys nothing, only a `v*` tag does")
    if push.get("tags") != ["v*"]:
        found.append(f"{name}: on.push.tags must be exactly [\"v*\"]")
    for key, value in {**(doc.get("env") or {}), "concurrency.group": doc.get("concurrency", {}).get("group")}.items():
        if re.search(r"\bdev\b", str(value)):
            found.append(f"{name}: env `{key}` names a dev environment: there is none")
    if doc.get("concurrency", {}).get("cancel-in-progress") is not False:
        found.append(f"{name}: concurrency: cancel-in-progress must be false (never cancel a run in progress)")
    found += check_no_gcp(name, doc)
    builds = build_steps(doc)
    if not builds:
        found.append(f"{name}: no job builds and pushes the image (docker/build-push-action)")
    for job_name, job, step in builds:
        found += check_login(name, job_name, job)
        found += check_build(name, doc, job_name, job, step)
    return found + check_summary(name, doc)


def check(workflows, actions):
    """Violations of `workflows` and `actions` (name → parsed YAML)."""
    found = []
    for name, doc in {**workflows, **actions}.items():
        found += check_permissions(name, doc)
        for step in steps_of(doc):
            found += check_step(name, step)
    for name, doc in workflows.items():
        found += check_siblings(name, doc)
        if name == "deploy.yml":
            found += check_deploy(name, doc)
    return found


def load(path, label, into, problems):
    """Parse one YAML file into `into[label]`, or record why it does not."""
    try:
        into[label] = yaml.safe_load(path.read_text())
    except yaml.YAMLError as err:
        problems.append(f"{label}: not valid YAML: {str(err).splitlines()[0]}")


def check_repository(root):
    """Load the repository's workflows and composite actions, then `check`."""
    workflows, actions, problems = {}, {}, []
    for path in sorted((root / ".github" / "workflows").glob("*.yml")):
        load(path, path.name, workflows, problems)
    for path in sorted((root / ".github" / "actions").glob("*/action.yml")):
        load(path, f"{path.parent.name}/action.yml", actions, problems)
    if not workflows and not problems:
        return ["no workflow found under .github/workflows"]
    return problems + check(workflows, actions)


if __name__ == "__main__":
    problems = check_repository(pathlib.Path(__file__).resolve().parent.parent)
    for line in problems:
        print(line, file=sys.stderr)
    print(f"check_workflows: {len(problems)} violation(s)")
    sys.exit(1 if problems else 0)

