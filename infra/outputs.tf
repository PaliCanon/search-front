output "secrets" {
  description = "The Secret Manager secrets to fill (no version is created by tofu)."
  value = {
    for k in var.envs : k => {
      api_token = google_secret_manager_secret.api_token[k].secret_id
    }
  }
}
