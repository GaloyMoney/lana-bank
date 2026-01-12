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

  lifecycle {
    replace_triggered_by = [google_bigquery_dataset.dataset.id]
  }
}

resource "google_bigquery_dataset_iam_member" "dataset_additional_owners" {
  for_each   = toset(local.additional_owners)
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dataset.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"

  lifecycle {
    replace_triggered_by = [google_bigquery_dataset.dataset.id]
  }
}

resource "google_bigquery_dataset" "dbt" {
  project                    = local.gcp_project
  dataset_id                 = local.dbt_dataset_name
  friendly_name              = "${local.name_prefix} dbt"
  description                = "dbt for ${local.name_prefix}"
  location                   = local.location
  delete_contents_on_destroy = true
}

resource "google_bigquery_dataset_iam_member" "dbt_owner" {
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dbt.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${google_service_account.bq_access_sa.email}"

  lifecycle {
    replace_triggered_by = [google_bigquery_dataset.dbt.id]
  }
}

resource "google_bigquery_dataset_iam_member" "dbt_additional_owners" {
  for_each   = toset(local.additional_owners)
  project    = local.gcp_project
  dataset_id = google_bigquery_dataset.dbt.dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"

  lifecycle {
    replace_triggered_by = [google_bigquery_dataset.dbt.id]
  }
}

resource "google_bigquery_dataset_access" "view_access" {
  dataset_id = google_bigquery_dataset.dataset.dataset_id
  project    = local.gcp_project
  dataset {
    dataset {
      project_id = local.gcp_project
      dataset_id = google_bigquery_dataset.dbt.dataset_id
    }
    target_types = ["VIEWS"]
  }

  lifecycle {
    replace_triggered_by = [
      google_bigquery_dataset.dataset.id,
      google_bigquery_dataset.dbt.id
    ]
  }
}
