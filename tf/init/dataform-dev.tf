resource "google_bigquery_dataset" "dataform_dev" {
  for_each = local.lava_dev
  project  = local.project

  dataset_id    = "dataform_${each.key}"
  friendly_name = "${each.key} dataform"
  description   = "Dataform playground for ${each.key}"
  location      = "EU"
}

resource "google_bigquery_dataset" "dataform_assertions_dev" {
  for_each = local.lava_dev
  project  = local.project

  dataset_id    = "dataform_assertions_${each.key}"
  friendly_name = "${each.key} assertions dataform"
  description   = "Dataform assertions for ${each.key}"
  location      = "EU"
}

resource "google_bigquery_dataset_iam_member" "dataform_assertions_dev" {
  for_each   = local.lava_dev
  project    = local.project
  dataset_id = google_bigquery_dataset.dataform_assertions_dev[each.key].dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"
}

resource "google_bigquery_dataset_iam_member" "dataform_assertions_sa" {
  for_each   = local.lava_dev
  project    = local.project
  dataset_id = google_bigquery_dataset.dataform_assertions_dev[each.key].dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${module.setup[each.key].service_account_email}"
}

resource "google_bigquery_dataset_iam_member" "dataform_dev" {
  for_each   = local.lava_dev
  project    = local.project
  dataset_id = google_bigquery_dataset.dataform_dev[each.key].dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "user:${each.value}"
}

resource "google_bigquery_dataset_iam_member" "dataform_sa" {
  for_each   = local.lava_dev
  project    = local.project
  dataset_id = google_bigquery_dataset.dataform_dev[each.key].dataset_id
  role       = "roles/bigquery.dataOwner"
  member     = "serviceAccount:${module.setup[each.key].service_account_email}"
}

resource "google_project_iam_member" "dev_jobuser" {
  for_each = local.lava_dev
  project  = local.project
  role     = "roles/bigquery.jobUser"
  member   = "user:${each.value}"
}

resource "google_project_iam_member" "read_session_user" {
  for_each = local.lava_dev
  project  = local.project
  role     = "roles/bigquery.readSessionUser"
  member   = "user:${each.value}"
}
