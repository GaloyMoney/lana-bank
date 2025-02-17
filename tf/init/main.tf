variable "git_token" {
  default   = "dummy"
  sensitive = true
}

variable "gcp_region" {
  type    = string
  default = "europe-west6"
}

locals {
  project                = "lana-dev-440721"
  tf_state_bucket_name   = "lana-dev-tf-state"
  objects_list_role_name = "lana_objects_list"

  justin = "justin@galoy.io"

  lana_dev = {
    jireva     = "jir@galoy.io",
    jcarter    = "justin@galoy.io"
    sv         = "sv@galoy.io"
    sandipndev = "sandipan@galoy.io"
    vaibhav    = "vaibhav@galoy.io"
    siddharth  = "siddharth@galoy.io"
    vindard    = "arvin@galoy.io"
    n          = "nb@galoy.io"
    krtk6160   = "krtk6160@galoy.io"
  }
}

module "setup" {
  source = "../bq-setup"

  for_each = local.lana_dev

  name_prefix = each.key

  additional_owners = [each.value, local.justin]
  gcp_project       = local.project
  gcp_region        = var.gcp_region
  git_token         = var.git_token
}

module "gha_setup" {
  source = "../bq-setup"

  name_prefix = "gha"

  additional_owners = [local.justin]
  gcp_project       = local.project
  gcp_region        = var.gcp_region
  git_token         = var.git_token
}

module "concourse_setup" {
  source = "../bq-setup"

  name_prefix = "concourse"

  additional_owners = [local.justin]
  gcp_project       = local.project
  gcp_region        = var.gcp_region
  git_token         = var.git_token
}

output "bq_dev_sa_keys_base64" {
  value     = { for key, value in module.setup : key => value.service_account_key_base64 }
  sensitive = true
}

output "holistics_dev_sa_keys_base64" {
  value     = { for key, value in module.setup : key => value.holistics_service_account_key_base64 }
  sensitive = true
}

output "bq_dev_sa_emails" {
  value = { for key, value in module.setup : key => value.service_account_email }
}

output "gha_sa_keys_base64" {
  value     = module.gha_setup.service_account_key_base64
  sensitive = true
}

output "concourse_sa_keys_base64" {
  value     = module.concourse_setup.service_account_key_base64
  sensitive = true
}

terraform {
  backend "gcs" {
    bucket = "lana-dev-tf-state"
    prefix = "lana-dev/setup"
  }
}
