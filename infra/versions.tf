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
    cloudflare = {
      source  = "cloudflare/cloudflare"
      version = "~> 4.52"
    }
  }
}

# Credentials come from the environment: gcloud auth application-default
# login, and CLOUDFLARE_API_TOKEN (zone DNS and Workers routes, account
# Workers scripts). Nothing is read from a file in this repository.
provider "google" {
  project = var.gcp_project
  region  = var.region
}

provider "cloudflare" {}
