provider "cala" {
  endpoint = "http://localhost:2252/graphql"
}

variable "bfx_key" {
  sensitive = true
  default   = "dummy_key"
}

variable "bfx_secret" {
  sensitive = true
  default   = "dummy_key"
}

terraform {
  required_providers {
    cala = {
      source  = "registry.terraform.io/galoymoney/cala"
      version = "0.0.17"
    }
  }
}
