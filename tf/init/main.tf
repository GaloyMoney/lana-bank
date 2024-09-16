variable "git_token" {
  default   = "dummy"
  sensitive = true
}

variable "gcp_region" {
  type    = string
  default = "europe-west6"
}

locals {
  project                = "cala-enterprise"
  tf_state_bucket_name   = "lava-bank-tf-state"
  objects_list_role_name = "lava_objects_list"

  justin = "justin@galoy.io"

  lava_dev = {
    jireva     = "jir@galoy.io",
    jcarter    = "justin@galoy.io"
    sv         = "sv@galoy.io"
    sandipndev = "sandipan@galoy.io"
    vaibhav    = "vaibhav@galoy.io"
    siddharth  = "siddharth@galoy.io"
    vindard    = "arvin@galoy.io"
  }
}

module "setup" {
  source = "../bq-setup"

  for_each = local.lava_dev

  name_prefix = each.key

  additional_owners = [each.value]
  dataform_dev_user = each.key
  gcp_project       = local.project
  gcp_region        = var.gcp_region
  git_token         = var.git_token
}

module "gha_setup" {
  source = "../bq-setup"

  name_prefix = "gha"

  dataform_dev_user = "gha"
  additional_owners = [local.justin]
  gcp_project       = local.project
  gcp_region        = var.gcp_region
  git_token         = var.git_token
}

output "bq_dev_sa_keys_base64" {
  value     = { for key, value in module.setup : key => value.service_account_key_base64 }
  sensitive = true
}

output "bq_dev_sa_emails" {
  value = { for key, value in module.setup : key => value.service_account_email }
}

output "gha_sa_keys_base64" {
  value     = module.gha_setup.service_account_key_base64
  sensitive = true
}

terraform {
  backend "gcs" {
    bucket = "lava-bank-tf-state"
    prefix = "lava-bank/setup"
  }
}

