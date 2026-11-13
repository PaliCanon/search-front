variable "gcp_project" {
  description = "The GCP project, shared with the site, the API and learn-pali."
  type        = string
  default     = "palicanon-app"
}

variable "region" {
  type    = string
  default = "europe-west1"
}

variable "bootstrap_image" {
  description = "Placeholder image for the first apply, before CI has pushed anything."
  type        = string
  default     = "us-docker.pkg.dev/cloudrun/container/hello"
}

variable "cloudflare_account_id" {
  type    = string
  default = "c0690c72f1b7c85737052c3a9ebac154"
}

# One entry per environment: prod is the only one (the dev environment was
# removed). The host is served by a Workers Custom Domain which issues the
# exact host's certificate (cloudflare.tf). `api_url` is the PaliCanon API this
# environment searches through (`SEARCH_API_URL`). Changing a host is an edit
# here and a tofu apply.
variable "envs" {
  type = map(object({
    host          = string
    max_instances = number
    api_url       = string
  }))
  default = {
    prod = {
      host          = "search.palicanon.app"
      max_instances = 3
      api_url       = "https://api.palicanon.app"
    }
  }
}
