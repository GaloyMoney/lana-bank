resource "google_service_account" "holistics" {
  project      = local.gcp_project
  account_id   = local.holistics_sa_name
  display_name = local.holistics_sa_name
}

resource "google_service_account_key" "holistics_key" {
  service_account_id = google_service_account.holistics.name
}

resource "google_project_iam_member" "holistics_job_user" {
  project = local.gcp_project
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${google_service_account.holistics.email}"
}

resource "google_bigquery_dataset_iam_member" "holistics_viewer" {
  project    = local.gcp_project
  dataset_id = local.dataform_dataset_name
  role       = "roles/bigquery.dataViewer"
  member     = "serviceAccount:${google_service_account.holistics.email}"
}

resource "google_project_iam_member" "holistics_meta_viewer" {
  project    = local.gcp_project
  role       = "roles/bigquery.metadataViewer"
  member     = "serviceAccount:${google_service_account.holistics.email}"
}
