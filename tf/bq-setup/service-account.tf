resource "google_service_account" "bq_access_sa" {
  project      = local.gcp_project
  account_id   = local.sa_account_id
  display_name = "Serviae Account for lava-bank BigQuery access"
}

resource "google_service_account_key" "bq_access_sa_key" {
  service_account_id = google_service_account.bq_access_sa.name
}

resource "google_project_iam_member" "project_browser" {
  project = local.gcp_project
  role    = "roles/dataform.serviceAgent"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_project_iam_member" "dataform_viewer" {
  project = local.gcp_project
  role    = "roles/dataform.viewer"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_project_iam_member" "dataform_agent" {
  project = local.gcp_project
  role    = "roles/dataform.serviceAgent"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_project_iam_member" "bq_jobuser" {
  project = local.gcp_project
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}
