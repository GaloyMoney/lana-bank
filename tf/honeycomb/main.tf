terraform {
  required_version = ">= 1.6.0"
  required_providers {
    honeycombio = {
      source  = "honeycombio/honeycombio"
      version = "~> 0.42.0"
    }
  }
}

variable "honeycomb_api_key" {
  type        = string
  description = "Honeycomb API key (set via TF_VAR_honeycomb_api_key)"
  sensitive   = true
}

variable "honeycomb_dataset" {
  type        = string
  description = "Honeycomb dataset name"
  default     = "lana-dev"
}

variable "name_prefix" {
  type        = string
  description = "Prefix for dashboard names"
  default     = "lana"
}

provider "honeycombio" {
  api_key = var.honeycomb_api_key
}

locals {
  name_prefix = var.name_prefix
}

