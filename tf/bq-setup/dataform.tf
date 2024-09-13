resource "google_project_iam_member" "service_account_impersonation" {
  project = local.gcp_project
  role    = "roles/iam.serviceAccountTokenCreator"
  member  = "serviceAccount:service-${data.google_project.project.number}@gcp-sa-dataform.iam.gserviceaccount.com"
}

resource "google_service_account_iam_member" "service_account_impersonation_target" {
  service_account_id = google_service_account.bq_access_sa.name
  role               = "roles/iam.serviceAccountTokenCreator"
  member             = "serviceAccount:service-${data.google_project.project.number}@gcp-sa-dataform.iam.gserviceaccount.com"
}

resource "google_project_iam_member" "dev_jobuser" {
  project = local.gcp_project
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_dataform_repository" "repository" {
  provider = google-beta

  project = local.gcp_project
  region  = local.gcp_region

  name         = local.dataform_repo_name
  display_name = "Dataform Repository for ${local.dataform_repo_name}"

  git_remote_settings {
    url                                 = "https://github.com/GaloyMoney/lava-bank.git"
    default_branch                      = local.dataform_git_branch
    authentication_token_secret_version = google_secret_manager_secret_version.version.id
  }
}

resource "google_dataform_repository_iam_member" "member" {
  provider = google-beta

  project    = google_dataform_repository.repository.project
  region     = google_dataform_repository.repository.region
  repository = google_dataform_repository.repository.name
  role       = "roles/owner"
  member     = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_dataform_repository_release_config" "release" {
  provider = google-beta

  project    = google_dataform_repository.repository.project
  region     = google_dataform_repository.repository.region
  repository = google_dataform_repository.repository.name

  name          = local.dataform_release_config_name
  git_commitish = local.dataform_git_commitish

  code_compilation_config {
    default_database = local.gcp_project
    default_schema   = "dataform"
    default_location = local.dataform_location
    assertion_schema = "dataform_assertions"
    schema_suffix    = local.name_prefix
    vars = {
      executionEnv = local.dataform_execution_env
      devUser = local.dataform_dev_user
    }
  }
}
resource "google_dataform_repository_workflow_config" "workflow" {
  provider = google-beta

  name = local.dataform_workflow_config_name

  project    = google_dataform_repository.repository.project
  region     = google_dataform_repository.repository.region
  repository = google_dataform_repository.repository.name

  release_config = google_dataform_repository_release_config.release.id

  invocation_config {
    transitive_dependencies_included         = true
    transitive_dependents_included           = true
    fully_refresh_incremental_tables_enabled = false
    service_account                          = google_service_account.bq_access_sa.email
  }
}
