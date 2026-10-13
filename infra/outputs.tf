output "urls" {
  description = "The Cloud Run URL of each environment (the Worker's ORIGIN). It answers 403 to anything but the Worker."
  value       = { for k, s in google_cloud_run_v2_service.app : k => s.uri }
}

output "hosts" {
  description = "The public address of each environment."
  value       = { for k, v in var.envs : k => "https://${v.host}" }
}

output "runtime_service_accounts" {
  description = "The identity each service runs as (secretAccessor on its two secrets, nothing else)."
  value       = { for k, a in google_service_account.app : k => a.email }
}

output "secrets" {
  description = "The Secret Manager secrets to fill (no version is created by tofu)."
  value = {
    for k in keys(var.envs) : k => {
      api_token     = google_secret_manager_secret.api_token[k].secret_id
      origin_secret = google_secret_manager_secret.origin[k].secret_id
    }
  }
}
