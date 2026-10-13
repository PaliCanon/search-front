# Shared with the site's infra (palisearch): read, never managed here.
data "google_service_account" "deployer" {
  account_id = "github-deployer"
}

# The Secret Manager API is enabled by the API's infra (`google_project_service`
# there); a project-wide switch is not declared twice.

# ---------------------------------------------------------------- secrets

# The PaliCanon API token this site searches with (`scripts/token` in the API
# repository issues it; its SHA-256 goes into `pali-api-tokens-<env>`). Tofu
# creates the secret and NEVER a version: the value would sit in its state.
# The user adds the version (README, "Order").
resource "google_secret_manager_secret" "api_token" {
  for_each  = var.envs
  secret_id = "search-front-api-token-${each.key}"
  replication {
    auto {}
  }
}

# The origin secret: the value the Worker sends as `x-origin-auth` and the
# service requires (403 without it). Created by `infra/scripts/origin-secret
# create <env>` (a random value that never leaves Secret Manager and the
# Worker's secret store), never by tofu.
resource "google_secret_manager_secret" "origin" {
  for_each  = var.envs
  secret_id = "search-front-origin-secret-${each.key}"
  replication {
    auto {}
  }
}

# ---------------------------------------------------------------- service account

# The identity the service runs as. One role per secret it reads, on that
# secret only: a role added here is a decision to write down, not a default.
resource "google_service_account" "app" {
  for_each     = var.envs
  account_id   = "search-front-${each.key}"
  display_name = "search-front (${each.key})"
}

resource "google_secret_manager_secret_iam_member" "app_reads_api_token" {
  for_each  = var.envs
  secret_id = google_secret_manager_secret.api_token[each.key].id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.app[each.key].email}"
}

resource "google_secret_manager_secret_iam_member" "app_reads_origin" {
  for_each  = var.envs
  secret_id = google_secret_manager_secret.origin[each.key].id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.app[each.key].email}"
}

# `gcloud run deploy` in CI acts as the runtime service account.
resource "google_service_account_iam_member" "deployer_acts_as" {
  for_each           = var.envs
  service_account_id = google_service_account.app[each.key].name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${data.google_service_account.deployer.email}"
}

# The rollout checks the new revision BEFORE it takes traffic, through its
# tagged URL, with the origin header (`scripts/rollout-check.sh`): CI reads
# the origin secret and nothing else. Consequence (task 007, ruling R3): every
# repository listed in `github_repos` of the site's infra can read this one
# secret.
resource "google_secret_manager_secret_iam_member" "deployer_reads_origin" {
  for_each  = var.envs
  secret_id = google_secret_manager_secret.origin[each.key].id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${data.google_service_account.deployer.email}"
}

# ---------------------------------------------------------------- the site

resource "google_cloud_run_v2_service" "app" {
  for_each = var.envs
  name     = "search-front-${each.key}"
  location = var.region
  ingress  = "INGRESS_TRAFFIC_ALL"

  template {
    service_account = google_service_account.app[each.key].email
    scaling {
      min_instance_count = 0
      max_instance_count = each.value.max_instances
    }
    timeout = "60s" # server-rendered pages, no streaming

    containers {
      image = var.bootstrap_image
      ports {
        container_port = 8080
      }
      resources {
        limits = {
          cpu    = "1"
          memory = "512Mi"
        }
        # CPU only while a request is served: scale to zero costs nothing.
        cpu_idle = true
      }
      # The Leptos settings (site address and root, PROD) are the image's own
      # ENV (Dockerfile), so they cannot drift from the files they describe.
      env {
        name  = "RUST_LOG"
        value = "info"
      }
      env {
        name  = "SEARCH_API_URL"
        value = each.value.api_url
      }
      # Both secrets are read when an instance starts: a new version needs a
      # new revision (README, "Rotation").
      env {
        name = "SEARCH_API_TOKEN"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.api_token[each.key].secret_id
            version = "latest"
          }
        }
      }
      env {
        name = "ORIGIN_SECRET"
        value_source {
          secret_key_ref {
            secret  = google_secret_manager_secret.origin[each.key].secret_id
            version = "latest"
          }
        }
      }
      # TCP, not HTTP (task 007, ruling R2): the service answers 403 to a
      # request without the origin header, and a Cloud Run HTTP probe header
      # is a plain string in the service resource, which would put the origin
      # secret in tofu state. The page itself is checked by the rollout, on
      # the tagged revision, before traffic moves.
      startup_probe {
        tcp_socket {
          port = 8080
        }
        # The margin covers a slow image pull on a cold start.
        # period_seconds × failure_threshold = 60 s for a cold pull.
        # gcloud requires timeout_seconds < period_seconds.
        period_seconds    = 5
        timeout_seconds   = 3
        failure_threshold = 12
      }
    }
  }

  lifecycle {
    # CI deploys images; tofu owns everything else (same rule as the API's
    # infra: the revision name is not ignored, or gcloud's is re-sent → 409).
    # `scaling` is ignored too, as there: a later `gcloud run` never fights
    # tofu over it, so max_instances in variables.tf is the value at
    # creation; changing it afterwards is a `tofu apply -replace` or a
    # gcloud update, written down in infra/README.md.
    ignore_changes = [template[0].containers[0].image, scaling, client, client_version]
  }
  depends_on = [
    google_secret_manager_secret_iam_member.app_reads_api_token,
    google_secret_manager_secret_iam_member.app_reads_origin,
    google_service_account_iam_member.deployer_acts_as,
  ]
}

# Invokable by anyone at the Cloud Run layer: the Worker has no Google
# identity to present. What keeps the `*.run.app` URL from being a way round
# Cloudflare is the service itself: it refuses (403) any request without the
# origin secret (task 007, design v4).
resource "google_cloud_run_v2_service_iam_member" "public" {
  for_each = var.envs
  name     = google_cloud_run_v2_service.app[each.key].name
  location = var.region
  role     = "roles/run.invoker"
  member   = "allUsers"
}
