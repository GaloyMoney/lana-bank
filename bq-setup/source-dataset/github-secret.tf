data "google_project" "project" {
  project_id = local.gcp_project
}

resource "google_secret_manager_secret" "git_token" {
  provider = google-beta
  project  = local.gcp_project

  secret_id = local.git_token_secret_id
  replication {
    auto {}
  }
}

resource "google_secret_manager_secret_version" "secret_version" {
  provider = google-beta
  secret   = google_secret_manager_secret.git_token.id

  secret_data = local.git_token
}

resource "google_secret_manager_secret_iam_binding" "git_token" {
  project   = google_secret_manager_secret.git_token.project
  secret_id = google_secret_manager_secret.git_token.secret_id
  role      = "roles/secretmanager.secretAccessor"
  members = [
    "serviceAccount:service-${data.google_project.project.number}@gcp-sa-dataform.iam.gserviceaccount.com",
  ]
}

resource "google_service_account_iam_member" "service_account_impersonation_target" {
  service_account_id = google_service_account.bq_access_sa.name
  role               = "roles/iam.serviceAccountTokenCreator"
  member             = "serviceAccount:service-${data.google_project.project.number}@gcp-sa-dataform.iam.gserviceaccount.com"
}
