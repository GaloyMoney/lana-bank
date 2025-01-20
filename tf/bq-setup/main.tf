variable "name_prefix" {}
variable "gcp_project" {}
variable "gcp_region" {}

variable "dataform_git_commitish" {
  type    = string
  default = ""
}
variable "dataform_dev_user" {
  type    = string
  default = "prod"
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
  holistics_sa_name      = "${var.name_prefix}-holistics"
  gcp_project            = var.gcp_project
  gcp_region             = var.gcp_region
  dataform_dev_user      = var.dataform_dev_user
  dataform_execution_env = local.dataform_dev_user == "prod" ? local.name_prefix : "lana-dev"
  dataform_location      = "EU"

  additional_owners = var.additional_owners

  dataset_id             = "${replace(local.name_prefix, "-", "_")}_dataset"
  sa_account_id          = replace("${var.name_prefix}-lana-bq-access", "_", "-")
  git_token              = var.git_token
  git_token_secret_id    = "${var.name_prefix}-git-token"
  dataform_keyring_name  = "${var.name_prefix}-dataform-keyring"
  dataform_key_name      = "${var.name_prefix}-dataform-key"
  dataform_git_branch    = "${var.name_prefix}-dataform"
  dataform_git_commitish = var.dataform_git_commitish != "" ? var.dataform_git_commitish : "${var.name_prefix}-dataform"


  dbt_dataset_name                 = replace("dbt_${local.name_prefix}", "-", "_")
  location                         = "US"
  dataform_dataset_name            = replace("dataform_${local.name_prefix}", "-", "_")
  dataform_assertions_dataset_name = replace("dataform_assertions_${local.name_prefix}", "-", "_")
  dataform_repo_name               = "${local.name_prefix}-repo"
  dataform_release_config_name     = "${var.name_prefix}-release"
  dataform_workflow_config_name    = "${var.name_prefix}-workflow"
  docs_bucket_name                 = "${var.name_prefix}-lana-documents"
}
