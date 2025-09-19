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

variable "grant_provisioning_permissions" {
  type        = bool
  default     = false
  description = "Grant elevated permissions to allow the service account to provision new infrastructure"
}

variable "force_destroy_bucket" {
  type        = bool
  default     = false
  description = "Allow force destruction of the storage bucket even if it contains objects"
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

  dbt_dataset_name     = replace("dbt_${local.name_prefix}", "-", "_")
  location             = "US"
  docs_bucket_name     = "${var.name_prefix}-lana-documents"
  force_destroy_bucket = var.force_destroy_bucket
}
