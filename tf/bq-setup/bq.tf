resource "google_bigquery_dataset" "dataset" {
  project                    = local.gcp_project
  dataset_id                 = local.dataset_id
  friendly_name              = "Dataset for lana-bank ${local.name_prefix}"
  description                = "Dataset for lana-bank ${local.name_prefix}"
  location                   = local.location
  delete_contents_on_destroy = true
}

resource "google_bigquery_dataset_iam_member" "dataset_owner_sa" {
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataset.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_bigquery_dataset_iam_member" "dataset_additional_owners" {
  for_each   = toset(local.additional_owners)
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataset.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"
}

resource "google_bigquery_dataset" "dataform" {
  project       = local.gcp_project
  dataset_id    = local.dataform_dataset_name
  friendly_name = "${local.name_prefix} dataform"
  description   = "Dataform for ${local.name_prefix}"
  location      = local.dataform_location
}

resource "google_bigquery_dataset_iam_member" "dataform_owner" {
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataform.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_bigquery_dataset_iam_member" "dataform_additional_owners" {
  for_each   = toset(local.additional_owners)
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataform.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"
}

resource "google_bigquery_dataset" "dataform_assertions" {
  project       = local.gcp_project
  dataset_id    = local.dataform_assertions_dataset_name
  friendly_name = "${local.name_prefix} assertions dataform"
  description   = "Dataform assertions for ${local.name_prefix}"
  location      = local.dataform_location
}

resource "google_bigquery_dataset_iam_member" "dataform_assertions_owner" {
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataform_assertions.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_bigquery_dataset_iam_member" "dataform_assertions_additional_owners" {
  for_each   = toset(local.additional_owners)
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataform_assertions.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"
}

resource "google_bigquery_dataset" "dbt" {
  project       = local.gcp_project
  dataset_id    = local.dbt_dataset_name
  friendly_name = "${local.name_prefix} dbt"
  description   = "dbt for ${local.name_prefix}"
  location      = local.location
}

resource "google_bigquery_dataset_iam_member" "dbt_owner" {
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dbt.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${google_service_account.bq_access_sa.email}"
}

resource "google_bigquery_dataset_iam_member" "dbt_additional_owners" {
  for_each   = toset(local.additional_owners)
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dbt.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"
}
