resource "google_service_account" "bq_access_sa" {
  project      = local.gcp_project
  account_id   = local.sa_account_id
  display_name = "Service Account for lana-bank BigQuery access"
}

resource "google_service_account_key" "bq_access_sa_key" {
  service_account_id = google_service_account.bq_access_sa.name
}

resource "google_project_iam_member" "bq_jobuser" {
  project = local.gcp_project
  role    = "roles/bigquery.jobUser"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_project_iam_member" "bq_resourceviewer" {
  project = local.gcp_project
  role    = "roles/bigquery.resourceViewer"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

# Elevated permissions for provisioning infrastructure (e.g., GitHub Actions)
resource "google_project_iam_member" "bq_admin" {
  count   = var.grant_provisioning_permissions ? 1 : 0
  project = local.gcp_project
  role    = "roles/bigquery.admin"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_project_iam_member" "sa_admin" {
  count   = var.grant_provisioning_permissions ? 1 : 0
  project = local.gcp_project
  role    = "roles/iam.serviceAccountAdmin"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_project_iam_member" "project_iam_admin" {
  count   = var.grant_provisioning_permissions ? 1 : 0
  project = local.gcp_project
  role    = "roles/resourcemanager.projectIamAdmin"
  member  = "serviceAccount:${google_service_account.bq_access_sa.email}"
}
