variable "name_prefix" {
  type    = string
  default = "dev_"
}

variable "deletion_protection" {
  type    = bool
  default = true
}

variable "gcp_region" {
  type    = string
  default = "europe-west6"
}

variable "setup_bq" {
  default = false
}

variable "sa_creds" {
  type    = string
  default = "dummy"
}

locals {
  name_prefix           = var.name_prefix
  gcp_region            = var.gcp_region
  setup_bq              = var.setup_bq
  git_token_secret_name = "${local.name_prefix}-git-token"
  dataset_id            = "${replace(local.name_prefix, "-", "_")}_dataset"
  bq_entity_tables = local.setup_bq ? [
    "user_events",
    "customer_events",
    "loan_events",
    "withdraw_events",
    "deposit_events",
    "credit_facility_events",
    "terms_template_events",
    "interest_accrual_events",
  ] : []
  deletion_protection = var.deletion_protection
  bq_applicant_table  = local.setup_bq ? "sumsub_applicants" : ""
  bq_price_cents_btc    = local.setup_bq ? "price_cents_btc" : ""

  service_account_creds = local.setup_bq ? jsondecode(base64decode(var.sa_creds)) : null
  project_id            = local.setup_bq ? local.service_account_creds.project_id : null
  sa_email              = local.setup_bq ? local.service_account_creds.client_email : null
  sa_member             = local.setup_bq ? "serviceAccount:${local.sa_email}" : null
}

terraform {
  required_providers {
    cala = {
      source  = "registry.terraform.io/galoymoney/cala"
      version = "0.0.20"
    }
  }
}
