variable "sa_creds" {
  type    = string
  default = "dummy"
}

variable "name_prefix" {
  type    = string
  default = "gha"
}

locals {
  setup_bq    = var.sa_creds != "dummy"
  name_prefix = var.name_prefix
  gcp_region  = "europe-west6"

  service_account_creds = local.setup_bq ? jsondecode(base64decode(var.sa_creds)) : null
  project_id            = local.setup_bq ? local.service_account_creds.project_id : null
  sa_email              = local.setup_bq ? local.service_account_creds.client_email : null
  sa_member             = local.setup_bq ? "serviceAccount:${local.sa_email}" : null
  dataset_id            = local.setup_bq ? "${replace(var.name_prefix, "-", "_")}_dataset" : null
}

provider "google" {
  project = local.project_id
}
