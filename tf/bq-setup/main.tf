variable "name_prefix" {}
variable "gcp_project" {}
variable "gcp_region" {}

variable "git_token" {
  sensitive = true
}

variable "additional_owners" {
  type    = list(string)
  default = []
}

locals {
  name_prefix       = var.name_prefix
  holistics_sa_name = "${var.name_prefix}-holistics"
  gcp_project       = var.gcp_project
  gcp_region        = var.gcp_region

  additional_owners = var.additional_owners

  dataset_id          = "${replace(local.name_prefix, "-", "_")}_dataset"
  sa_account_id       = replace("${var.name_prefix}-lana-bq-access", "_", "-")
  git_token           = var.git_token
  git_token_secret_id = "${var.name_prefix}-git-token"

  dbt_dataset_name = replace("dbt_${local.name_prefix}", "-", "_")
  location         = "US"
  docs_bucket_name = "${var.name_prefix}-lana-documents"
}
