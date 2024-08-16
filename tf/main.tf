provider "cala" {
  endpoint = "http://localhost:2252/graphql"
}

module "setup" {
  source = "./lava-setup"
}

terraform {
  required_providers {
    cala = {
      source  = "registry.terraform.io/galoymoney/cala"
      version = "0.0.19"
    }
  }
}
