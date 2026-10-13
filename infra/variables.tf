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

# One entry per environment. The dev host is search-dev.palicanon.app: one
# level under the zone, inside Universal SSL's `*.palicanon.app`; it is
# served by a Workers Custom Domain all the same (cloudflare.tf), as
# learn-pali's. `api_url` is the PaliCanon API this environment searches
# through (`SEARCH_API_URL`). Changing a host is an edit here and a tofu apply.
#
# The default holds `dev` ONLY: the first apply must not stand up a public
# prod host before dev has deployed and been checked from CI. Prod is added
# afterwards, with `-var-file` or by uncommenting the block below (README,
# "Order"), then a second apply, with its own two secrets.
variable "envs" {
  type = map(object({
    host          = string
    max_instances = number
    api_url       = string
  }))
  default = {
    dev = {
      host          = "search-dev.palicanon.app"
      max_instances = 2
      api_url       = "https://api-dev.palicanon.app"
    }
    # prod = {
    #   host          = "search.palicanon.app"
    #   max_instances = 3
    #   api_url       = "https://api.palicanon.app"
    # }
  }
}
