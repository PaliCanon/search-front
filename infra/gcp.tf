# ---------------------------------------------------------------- secrets

# The PaliCanon API token this site searches with (`scripts/token` in the API
# repository issues it; its SHA-256 goes into `pali-api-tokens-<env>`). Tofu
# creates the secret and NEVER a version: the value would sit in its state.
# The user adds the version (README, "Order"). The container on the user's
# server (Lsh0x/lsh-tech, stack palicanon/prod) reads it from Secret Manager.
resource "google_secret_manager_secret" "api_token" {
  for_each  = var.envs
  secret_id = "search-front-api-token-${each.key}"
  replication {
    auto {}
  }
}

# Cloud Run is gone (the service, its runtime service account and their
# bindings, the origin secret that only guarded it, the deployer's grants):
# prod runs on the server, images come from GHCR (deploy.yml). The Secret
# Manager API is enabled by the API's infra (`google_project_service` there).
