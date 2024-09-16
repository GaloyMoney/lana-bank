variable "name_prefix" {}
variable "gcp_project" {}
variable "gcp_region" {}

variable "dataform_git_commitish" {
  type    = string
  default = ""
}
variable "dataform_dev_user" {
  type    = string
  default = ""
}

variable "git_token" {
  sensitive = true
}

variable "additional_owners" {
  type    = list(string)
  default = []
}

locals {
  name_prefix            = var.name_prefix
  gcp_project            = var.gcp_project
  gcp_region             = var.gcp_region
  dataform_dev_user      = var.dataform_dev_user
  dataform_execution_env = local.dataform_dev_user == "" ? local.name_prefix : "volcano-dev"
  dataform_location      = "EU"

  additional_owners = var.additional_owners

  dataset_id             = "${replace(local.name_prefix, "-", "_")}_dataset"
  sa_account_id          = replace("${var.name_prefix}-lava-bq-access", "_", "-")
  git_token              = var.git_token
  git_token_secret_id    = "${var.name_prefix}-git-token"
  dataform_keyring_name  = "${var.name_prefix}-dataform-keyring"
  dataform_key_name      = "${var.name_prefix}-dataform-key"
  dataform_git_branch    = "${var.name_prefix}-dataform"
  dataform_git_commitish = var.dataform_git_commitish != "" ? var.dataform_git_commitish : "${var.name_prefix}-dataform"


  dataform_repo_name            = "${local.name_prefix}-repo"
  dataform_release_config_name  = "${var.name_prefix}-release"
  dataform_workflow_config_name = "${var.name_prefix}-workflow"
}

output "service_account_key_base64" {
  value = google_service_account_key.bq_access_sa_key.private_key
}

output "service_account_email" {
  value = google_service_account.bq_access_sa.email
}

output "dataset_id" {
  value = google_bigquery_dataset.dataset.dataset_id
}
