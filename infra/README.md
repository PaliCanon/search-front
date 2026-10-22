# infra

search-front's own resources, in its own tofu state
(`gs://palicanon-app-tofu-state`, prefix `search-front`), inside the GCP
project it shares with the site, the API and learn-pali (`palicanon-app`,
`europe-west1`). Per environment (the map `envs` in `variables.tf`: `dev`
first, `prod` added later, see "Order"):

| What | File | Notes |
| --- | --- | --- |
| Runtime service account `search-front-<env>` | `gcp.tf` | **One role per secret it reads**, on that secret only: `secretAccessor` on the API token and on the origin secret. Nothing else. |
| Secret `search-front-api-token-<env>` | `gcp.tf` | The PaliCanon API token this site searches with. Tofu creates the secret and **never a version** (the value would sit in its state). |
| Secret `search-front-origin-secret-<env>` | `gcp.tf` | The value the Worker sends as `x-origin-auth` and the service requires. Same rule: no version in tofu; `scripts/origin-secret` makes it. |
| The deployer's rights | `gcp.tf` | `github-deployer` is read as a data source (the site's infra owns it), never managed here. It may **act as** the runtime account and **read the origin secret** (see "R3"). |
| Cloud Run service `search-front-<env>` | `gcp.tf` | min 0, max 2 (dev) / 3 (prod), 1 CPU, 512Mi, CPU only while a request is served, public at the Cloud Run layer. `SEARCH_API_URL` is a value; `SEARCH_API_TOKEN` and `ORIGIN_SECRET` come from the two secrets (`latest`, read when an instance starts). Startup probe: **TCP 8080** (see "R2"). |
| A Worker relaying to Cloud Run | `cloudflare.tf`, `worker/proxy.js` | The Worker's `ORIGIN` is the Cloud Run URL and its `PUBLIC_ORIGIN` the host's own `https://` origin (the target of a rewritten `Location`, never the request's Host; both answer 500 when missing); it adds `x-origin-auth` from its secret `ORIGIN_AUTH` and answers 500 without it. Tested with `node --test 'infra/worker/*.test.mjs'`. |
| A Workers Custom Domain per host | `cloudflare.tf` | `search.dev.palicanon.app` (dev), `search.palicanon.app` (prod, added later). Cloudflare creates the DNS record and the certificate. |
| `scripts/origin-secret` | `scripts/` | `create`, `push`, `check` the origin secret; tested with stub `gcloud`/`curl` (`bash infra/scripts/origin-secret.test.sh`). |

CI owns the image: tofu ignores `image`, the revision name, the container
`env`, `scaling`, `client` and `client_version` of the service (`lifecycle.ignore_changes`, the
API's rule), so a deploy never shows up as drift, and CI never touches the
secrets. The deploy writes `SEARCH_FRONT_ENV` and `SEARCH_FRONT_SHA` into the env
(`gcloud run deploy --update-env-vars`, the version badge reads them), never
tofu. The `env` ignore must be applied (`tofu apply`) before the first deploy
that writes the env, or the next apply strips both variables and the badge
reads "dev" with no diff shown. The consequence of ignoring `env`: a changed
`RUST_LOG` or `SEARCH_API_TOKEN` reference is not applied to an existing service;
change it with `gcloud run services update` or `tofu apply -replace`. A
`tofu apply -replace` of the service also drops `SEARCH_FRONT_ENV` and
`SEARCH_FRONT_SHA`: the badge reads `dev` until the next deploy. The next push to
main writes them again (`main · <sha7>`); a prod tag has no manual trigger
(`deploy.yml` runs on a push to main or a `v*` tag), so it is restored by
`gcloud run deploy ... --update-env-vars SEARCH_FRONT_ENV=v0.1.8,SEARCH_FRONT_SHA=`
by hand, or by deleting and re-pushing the tag. The deploy uses `--update-env-vars` and
never `--set-env-vars`: the set variant replaces the whole env and wipes
`RUST_LOG`, `SEARCH_API_URL` and the secret references, so every request would be
refused. The consequence of ignoring `scaling`: a changed `max_instances`
is not applied to an existing service; change it with
`gcloud run services update --max-instances` and keep `variables.tf` in step.

The Artifact Registry repository `palicanon`, the GitHub deploy identity
(workload identity federation), the Secret Manager API and the DNS zone
belong to the site's infra, the API's infra and `platform`: only used here.

## The origin rule

The service answers **403 to any request without `x-origin-auth`** equal to
its `ORIGIN_SECRET` (constant-time comparison), so the `*.run.app` URL is no
way round Cloudflare (its WAF, its per-IP limit): only the Worker holds the
value. The same value lives in two places, set by `scripts/origin-secret`:
Secret Manager (the service reads it) and the Worker's secret store (the
Worker sends it). Neither is in tofu state.

### First deploy: the Worker holds the secret first

The service refuses any request without the value as soon as a revision that
requires it serves, so the Worker must already hold it. In the order of "Only
the user" below: `origin-secret create <env>` (step 6), `tofu apply` (the
Worker, step 8), `origin-secret push <env>` and `check` (step 9), and only
then the first `deploy.yml` run (step 10). A deploy before the `push` fails
its public-host check with the Worker's 500: `deploy.yml` fails naming
`origin-secret push` and leaves the traffic where it is (the rollout is not
at fault; the new revision serves, the Worker cannot reach it yet). Run the
`push`, then re-run the job.

### Rotation

The value is read when an instance starts (the service) and when
`origin-secret push` runs (the Worker); there is no overlap, so there is a
short window in which the two disagree.

- **dev**: add the new version of `search-front-origin-secret-dev`, run
  `origin-secret push dev` (the Worker now sends the new value), then
  **start a new revision** that reads it. `deploy.yml` skips an image that
  already serves, so a rotation is not a CI run: force the revision with
  `gcloud run services update search-front-dev --region europe-west1
  --revision-suffix rot-<date>` (not yet exercised: confirm on the first
  rotation). Until that revision serves, the service answers the Worker with
  403: a short refusal window, accepted on dev.
- **prod**: not rotated this way. A refusal window on the public site is not
  accepted; prod waits for a **two-value rotation** (the service accepts the
  old and the new value for a while, the Worker switches in between), a
  follow-up task. Until it exists, prod's value is set once and left.

## Order: dev first, prod later

`var.envs` holds **dev only** by default. The first apply therefore stands up
`search-front-dev` and `search.dev.palicanon.app` and nothing public in prod.

## Only the user, in order (dev; prod repeats 5-10 later)

Nothing here has been applied yet (task 007 wrote it, did not run it).

1. Create the GitHub App in the PaliCanon organisation (permission: Contents
   read) and install it on `ui-lib`, `api` and `search-front`. Fallback: a
   fine-grained PAT of a machine user on the same three repositories.
2. Push `ui-lib` main, `api` main (the API client is in it) and
   `search-front`. Repository secrets on `search-front`: `APP_ID`,
   `APP_PRIVATE_KEY`; repository variables `WIF_PROVIDER`, `DEPLOYER_SA`,
   `GCP_PROJECT` (as learn-pali's).
3. palisearch: merge `infra/search-front-deploy-identity`, then the targeted
   apply of the workload identity (the provider's `attribute_condition` and
   `google_service_account_iam_member.deployer_wif["PaliCanon/search-front"]`).
4. Log in, and read the Cloudflare token from the keychain (never typed or
   printed):

   ```sh
   gcloud auth application-default login
   export CLOUDFLARE_API_TOKEN=$(security find-generic-password -a lsh -s cloudflare-api-token-palicanon -w)
   ```

5. In `infra/`: `tofu init`, then a **targeted** apply of the two secrets,
   the runtime service account and its IAM members (the Cloud Run service
   cannot start before the secrets have a version):

   ```sh
   cd infra
   tofu init
   tofu apply \
     -target=google_secret_manager_secret.api_token \
     -target=google_secret_manager_secret.origin \
     -target=google_service_account.app \
     -target=google_secret_manager_secret_iam_member.app_reads_api_token \
     -target=google_secret_manager_secret_iam_member.app_reads_origin \
     -target=google_secret_manager_secret_iam_member.deployer_reads_origin \
     -target=google_service_account_iam_member.deployer_acts_as
   ```

6. Generate the origin secret (stored in Secret Manager; nothing secret is
   printed):

   ```sh
   infra/scripts/origin-secret create dev
   ```

7. In the API repository, issue the site token and store it, the value never
   typed, shown or saved elsewhere:

   ```sh
   scripts/token new search-front --env dev \
     | gcloud secrets versions add search-front-api-token-dev --project palicanon-app --data-file=-
   ```

8. The rest: `tofu apply` (Cloud Run on the placeholder image, the Worker,
   the Custom Domain). The first apply may take a few minutes after the
   Custom Domain is created: Cloudflare issues the host's certificate then.
9. Give the Worker its secret and check it:

   ```sh
   infra/scripts/origin-secret push dev
   infra/scripts/origin-secret check dev   # prints OK
   ```

10. Push `search-front` main: `deploy.yml` rolls dev; look at
    https://search.dev.palicanon.app.

The services start with the **bootstrap image**
(`us-docker.pkg.dev/cloudrun/container/hello`, `var.bootstrap_image`), which
answers on 8080 and so passes the TCP startup probe; it does not check the
origin header, so `check` says OK before the first real deploy proves
nothing about the guard (the rollout does).

## Prod

After dev has deployed from CI and been checked: copy `prod.tfvars.example`
to `prod.tfvars` (git-ignored), and repeat steps 5-9 with `prod`
(`origin-secret create prod`, its own API token with `--env prod`,
`tofu apply -var-file=prod.tfvars`, `origin-secret push prod`). A `v*` tag
then deploys the real image.

## Rulings that shape this directory (task 007)

- **R2, the TCP probe.** The service refuses a request without the origin
  header, a Cloud Run HTTP probe header is a plain string in the service
  resource, and the origin secret would then sit in the tofu state and in the
  revision JSON. So the startup probe is TCP on 8080; the HTTP check of `/`
  runs in the rollout (`scripts/rollout-check.sh`) on the tagged revision,
  before traffic moves, with the header read from Secret Manager.
- **R3, the deployer reads the origin secret.** The pre-traffic check needs
  the header, so `github-deployer` has `secretAccessor` on
  `search-front-origin-secret-<env>`, that secret only. Consequence: any
  repository listed in `github_repos` of the site's infra (workload identity)
  can read that one secret. It does not read the API token.
- **R4, a re-uploaded Worker can lose its secret.** The Cloudflare provider
  has no `keep_bindings`: a `tofu apply` that changes `worker/proxy.js`, the
  compatibility date or `ORIGIN` re-uploads the script, and Cloudflare
  replaces bindings on upload unless they are kept. This was **not probed**
  (it needs a Cloudflare write). Rule: **run `infra/scripts/origin-secret
  push <env>` after any apply that touches the Worker.** The Worker answers
  500 when its secret is missing, so a drop is loud (the rollout's check
  through the Worker host fails, and `origin-secret check` says why).
  Declaring the secret in tofu (`cloudflare_workers_secret`) would put the
  value in the state: ruled out.
- **R6, Bot Fight.** The check from a GitHub runner to
  `search.dev.palicanon.app` after traffic can meet Cloudflare's Bot Fight
  Mode or a managed challenge (HTTP 403 with `cf-mitigated: challenge`). The
  script tells it from the service's own 403 (empty body, no `cf-mitigated`).
  If it happens, the decision is the user's: a WAF skip rule for the runner,
  or drop that one check. Not decided here.

## Not done here

- **Cloudflare Access on dev.** Like learn-pali's, `search.dev.palicanon.app`
  is public until an Access application is added for it (dashboard, or tofu
  once the token has the Access scope). The origin rule keeps the Cloud Run
  URL closed; Access would gate the visitors.
- The hosts (`search.dev.palicanon.app`, `search.palicanon.app`) are the
  user's (2026-10-11).
