# infra

search-front's own resources, in its own tofu state
(`gs://palicanon-app-tofu-state`, prefix `search-front`), inside the GCP
project it shares with the site, the API and learn-pali (`palicanon-app`).

Prod runs on the user's server (ssh.lsh.tech, repository `Lsh0x/lsh-tech`,
stack `palicanon/prod`): the container is pulled **by digest** from GHCR
(`ghcr.io/palicanon/search-front`) and sits behind a Cloudflare tunnel. Cloud
Run, its service account, the Worker and the origin secret are gone (2026-11-13).
What is left here:

| What | File | Notes |
| --- | --- | --- |
| Secret `search-front-api-token-prod` | `gcp.tf` | The PaliCanon API token this site searches with. The container on the server reads it from Secret Manager. Tofu creates the secret and **never a version** (the value would sit in its state). |

The Secret Manager API, the DNS zone and the Cloudflare account belong to the
site's infra, the API's infra and `platform`: not used here. There is no
Cloudflare provider: no Cloudflare resource is left in this state.

## The image

A `v*` tag runs `.github/workflows/deploy.yml`: it builds the image (no
credential, see `Dockerfile`) and pushes `ghcr.io/palicanon/search-front:h-<hash>`
(the hash of `scripts/image-hash.sh`: an existing tag skips the build) and
`:<release tag>`. The run's summary gives `ghcr.io/palicanon/search-front@sha256:...`:
the stack is updated from it by hand.

## The API token

1. Issue a token in the API repository (`scripts/token`), put its SHA-256 in
   `pali-api-tokens-prod` (API's infra).
2. Add the value as a version of `search-front-api-token-prod`:
   `printf %s "$TOKEN" | gcloud secrets versions add search-front-api-token-prod --project palicanon-app --data-file=-`
3. Restart the container on the server (it reads the secret at start).

## Removing Cloud Run (the order)

The Cloud Run service has `deletion_protection = true` in the state, so tofu
cannot destroy it. Once the server serves everything, in this order:

1. `gcloud run services delete search-front-prod --project palicanon-app --region europe-west1`
2. `cd infra && tofu apply`: it destroys the service account, the origin
   secret and the bindings, and drops the Cloudflare data source from the
   state. Read the plan first: nothing else may go, and
   `google_secret_manager_secret.api_token["prod"]` must stay.
