terraform {
  required_version = ">= 1.8"

  # Its own state, next to the site's (prefix "palicanon"), the API's
  # (prefix "pali-canon-api") and learn-pali's in the same bucket.
  backend "gcs" {
    bucket = "palicanon-app-tofu-state"
    prefix = "search-front"
  }

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
  }
}

# Credentials come from the environment: gcloud auth application-default
# login. Nothing is read from a file in this repository. No Cloudflare
# provider: no Cloudflare resource is left in this state (the Worker went with
# task 035, the zone is the site's).
provider "google" {
  project = var.gcp_project
  region  = var.region
}
