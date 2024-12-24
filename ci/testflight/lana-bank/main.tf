variable "testflight_namespace" {}

locals {
  cluster_name     = "galoy-staging-cluster"
  cluster_location = "us-east1"
  gcp_project      = "galoystaging"

  smoketest_namespace  = "galoy-staging-smoketest"
  cala_namespace       = var.testflight_namespace
  testflight_namespace = var.testflight_namespace
}

resource "kubernetes_namespace" "testflight" {
  metadata {
    name = local.testflight_namespace
  }
}

resource "random_password" "postgresql" {
  length  = 20
  special = false
}

provider "cala" {
  endpoint = "http://cala.${kubernetes_namespace.testflight.metadata[0].name}.svc.cluster.local:2252/graphql"
}

module "setup" {
  source = "./chart/tf/cala-setup"

  depends_on = [helm_release.cala]
}

resource "kubernetes_secret" "lana_bank" {
  metadata {
    name      = "lana-bank"
    namespace = kubernetes_namespace.testflight.metadata[0].name
  }

  data = {
    pg-user-pw : random_password.postgresql.result
    pg-con : "postgres://lana-bank:${random_password.postgresql.result}@lana-bank-postgresql:5432/lana-bank"
    bq-service-account-base64 : "eyAgInR5cGUiOiAic2VydmljZV9hY2NvdW50IiwgICJwcm9qZWN0X2lkIjogImFiY19hcHAiLCAgInByaXZhdGVfa2V5X2lkIjogImFiYyIsICAicHJpdmF0ZV9rZXkiOiAiLS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tXG5NSUlFdmdJQkFEQU5CZ2txaGtpRzl3MEJBUUVGQUFTQ0JLZ3dnZ1NrQWdFQUFvSUJBUURZM0U4bzFORUZjak1NXG5IVy81WmZGSncyOS84TkVxcFZpTmpRSXg5NVh4NUtEdEorblduOStPVzB1cXNTcUtsS0doQWRBbytRNmJqeDJjXG51WFZzWFR1N1hyWlVZNUtsdHZqOTREdlVhMXdqTlhzNjA2ci9SeFdUSjU4YmZkQytnTEx4QmZHbkI2Q3dLMFlRXG54bmZwak5ia1VmVlZ6TzBNUUQ3VVAwSGw1WmNZMFB1dnhkL3lIdU9OUW4vcklBaWVUSEgxcHFnVyt6ckgveTNjXG41OUlHVGhDOVBQdHVnSTllYThSU25WajNQV3oxYlgyVWtDRHB5OUlSaDlMekpMYVlZWDlSVWQ3KytkVUxVbGF0XG5BYVhCaDFVNmVtVUR6aHJJc2dBcGpEVnRpbU9QYm1RV21YMVM2MG1xUWlrUnBWWVo4dStOREQrTE53Ky9Fb3ZuXG54Q2oyWTN6MUFnTUJBQUVDZ2dFQVdEQnpvcU8xSXZWWGpCQTJscUlkMTBUNmhYbU4zajFpZnlIK2FBcUsrRlZsXG5HanlXakRqMHhXUWNKOXluYzdiUTZmU2VUZU5HelAwTTZrekRVMSt3NkZneVpxd2RtWFdJMlZtRWl6Ump3aysvXG4vdUxRVWNMN0k1NUR4bjdLVW9acy9yWlBtUUR4bUdMb3VlNjBHZzZ6M3lMelZjS2lEYzdjbmh6aGRCZ0RjOHZkXG5Rb3JOQWxxR1BSbm0zRXFLUTZWUXA2ZnlRbUNBeHJyNDVrc3BSWE5MZGRhdDNBTXN1cUltRGtxR0tCbUYzUTF5XG54V0dlODFMcGhVaVJxdnFieVVsaDZjZFNaOHBMQnBjOW0wYzNxV1BLczlwYXFCSXZnVVBsdk9aTXFlYzZ4NFM2XG5DaGJka2tUUkxuYnNScjBZZy9uRGVFUGxraFJCaGFzWHB4cE1VQmdQeXdLQmdRRHMyYXhOa0ZqYlU5NHVYdmQ1XG56blVoRFZ4UEZCdXh5VUh0c0pOcVc0cC91akxOaW1HZXQ1RS9ZdGhDblFlQzJQM1ltN2MzZml6NjhhbU02aGlBXG5Pblc3SFlQWitqS0ZuZWZwQXRqeU9PczQ2QWtmdEVnMDdUOVhqd1dOUHQ4KzhsMERZYXdQb0pnYk01aUUwTDJPXG54OFRVMVZzNG1YYytxbDlGOTBHekkweDNWd0tCZ1FEcVpPT3FXdzNoVG5OVDA3SXhxbm1kM2R1Z1Y5UzdlVzZvXG5VOU9vVWdKQjRyWVRwRyt5RnFOcWJSVDhia3gzN2lLQk1FUmVwcHFvbk9xR200d3R1UlI2TFNMbGdjSVU5SXd4XG55ZkgxMlVXcVZtRlNIc2daRnFNL2NLM3dHZXYzOGgxV0JJT3gzL2RqS243QmRsS1ZoOGtXeXg2dUM4Ym1WK0U2XG5Pb0swdkpENmt3S0JnSEF5U09uUk9CWmxxemtpS1c4Yyt1VTJWQVR0ekpTeWRyV20wSjR3VVBKaWZOQmEvaFZXXG5kY3FtQXpYQzl4em50NUFWYTN3eEhCT2Z5S2FFK2lnOENTc2pOeU5aM3ZibXIwWDA0Rm9WMW05MWsyVGVYTm9kXG5qTVRvYmtQVGhhTm00ZUxKTU4yU1FKdWFIR1RHRVJXQzBsM1QxOHQrL3pyRE1EQ1BpU0xYMU5BdkFvR0JBTjFUXG5WTEpZZGp2SU14ZjFibTU5VlljZXBiSzdITEhGa1JxNnhNSk1aYnRHMHJ5cmFaalV6WXZCNHE0VmpIazJVRGlDXG5saHgxM3RYV0RaSDdNSnRBQnpqeWcrQUk3WFdTRVFzMmNCWEFDb3MwTTRNeWM2bFUrZUwraUErT3VvVU9obXJoXG5xbVQ4WVlHdTc2L0lCV1VTcVd1dmNwSFBwd2w3ODcxaTRHYS9JM3FuQW9HQkFOTmtLQWNNb2VBYkpRSzdhL1JuXG53UEVKQitkUGdORElhYm9Bc2gxblpoVmhONWN2ZHZDV3VFWWdPR0NQUUxZUUYwem1UTGNNK3NWeE9ZZ2Z5OG1WXG5mYk5nUGdzUDV4bXU2ZHcyQ09CS2R0b3p3MEhyV1NSakFDZDFONHlHdTc1K3dQQ2NYL2dRYXJjalJjWFhaZUVhXG5OdEJMU2ZjcVBVTHFEK2g3YnI5bEVKaW9cbi0tLS0tRU5EIFBSSVZBVEUgS0VZLS0tLS1cbiIsICAiY2xpZW50X2VtYWlsIjogIjEyMy1hYmNAZGV2ZWxvcGVyLmdzZXJ2aWNlYWNjb3VudC5jb20iLCAgImNsaWVudF9pZCI6ICIxMjMtYWJjLmFwcHMuZ29vZ2xldXNlcmNvbnRlbnQuY29tIiwgICJhdXRoX3VyaSI6ICJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20vby9vYXV0aDIvYXV0aCIsICAidG9rZW5fdXJpIjogImh0dHA6Ly9sb2NhbGhvc3Q6ODA4MS90b2tlbiJ9"
    next-auth-secret : "dummydummydummy"
    next-auth-admin-pg-con : "postgres://lana-bank:${random_password.postgresql.result}@lana-bank-postgresql:5432/lana-bank"
    smtp-uri : "dummy"
    sumsub-key : "dummy"
    sumsub-secret : "dummy"
  }
}

resource "helm_release" "postgresql" {
  name       = "postgresql"
  repository = "https://charts.bitnami.com/bitnami"
  chart      = "postgresql"
  version    = "11.9.13"
  namespace  = kubernetes_namespace.testflight.metadata[0].name

  values = [
    file("${path.module}/postgresql-values.yml")
  ]
}

resource "jose_keyset" "oathkeeper" {}

resource "kubernetes_secret" "oathkeeper" {
  metadata {
    name      = "lana-bank-oathkeeper"
    namespace = kubernetes_namespace.testflight.metadata[0].name
  }

  data = {
    "mutator.id_token.jwks.json" = jsonencode({
      keys = [jsondecode(jose_keyset.oathkeeper.private_key)]
    })
  }
}

resource "helm_release" "lana_bank" {
  name      = "lana-bank"
  chart     = "${path.module}/chart"
  namespace = kubernetes_namespace.testflight.metadata[0].name

  values = [
    templatefile("${path.module}/testflight-values.yml.tmpl", {
      cala_namespace : local.cala_namespace
    })
  ]

  depends_on = [
    kubernetes_secret.lana_bank,
    kubernetes_secret.oathkeeper,
    helm_release.cala
  ]

  dependency_update = true
}

resource "kubernetes_secret" "smoketest" {
  metadata {
    name      = local.testflight_namespace
    namespace = local.smoketest_namespace
  }
  data = {
    kratos_database_url    = "postgresql://kratos-pg:kratos-pg@postgresql.${local.testflight_namespace}.svc.cluster.local/kratos-pg"
    kratos_public_endpoint = "http://lana-bank-kratos-public.${local.testflight_namespace}.svc.cluster.local"
    server_endpoint        = "http://lana-bank-public.${local.testflight_namespace}.svc.cluster.local"
    admin_endpoint         = "http://lana-bank-admin.${local.testflight_namespace}.svc.cluster.local"
    cala_endpoint          = "http://cala.${local.testflight_namespace}.svc.cluster.local"
  }
}

resource "random_password" "cala_postgresql" {
  length  = 20
  special = false
}

resource "kubernetes_secret" "cala" {
  metadata {
    name      = "cala"
    namespace = kubernetes_namespace.testflight.metadata[0].name
  }

  data = {
    pg-user-pw : random_password.cala_postgresql.result
    pg-con : "postgres://cala:${random_password.cala_postgresql.result}@cala-postgresql:5432/cala"
  }
}

resource "helm_release" "cala" {
  name      = "cala"
  chart     = "${path.module}/cala/vendor"
  namespace = kubernetes_namespace.testflight.metadata[0].name

  values = [
    file("${path.module}/cala-values.yml.tmpl")
  ]

  depends_on = [kubernetes_secret.cala]

  dependency_update = true
}

data "google_container_cluster" "primary" {
  project  = local.gcp_project
  name     = local.cluster_name
  location = local.cluster_location
}

data "google_client_config" "default" {
  provider = google-beta
}

provider "kubernetes" {
  host                   = "https://${data.google_container_cluster.primary.private_cluster_config.0.private_endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(data.google_container_cluster.primary.master_auth.0.cluster_ca_certificate)
}

provider "kubernetes-alpha" {
  host                   = "https://${data.google_container_cluster.primary.private_cluster_config.0.private_endpoint}"
  token                  = data.google_client_config.default.access_token
  cluster_ca_certificate = base64decode(data.google_container_cluster.primary.master_auth.0.cluster_ca_certificate)
}

provider "helm" {
  kubernetes {
    host                   = "https://${data.google_container_cluster.primary.private_cluster_config.0.private_endpoint}"
    token                  = data.google_client_config.default.access_token
    cluster_ca_certificate = base64decode(data.google_container_cluster.primary.master_auth.0.cluster_ca_certificate)
  }
}

terraform {
  required_providers {
    jose = {
      source  = "bluemill/jose"
      version = "1.0.0"
    }
    cala = {
      source  = "registry.terraform.io/galoymoney/cala"
      version = "0.0.20"
    }
  }
}
