variable "gcp_project" {
  description = "The GCP project, shared with the site, the API and learn-pali."
  type        = string
  default     = "palicanon-app"
}

variable "region" {
  type    = string
  default = "europe-west1"
}

# prod is the only environment (the dev environment was removed). It only
# names the API token secret; the host and the API URL are the stack's
# (Lsh0x/lsh-tech, stacks/palicanon/prod).
variable "envs" {
  type    = set(string)
  default = ["prod"]
}
