resource "google_storage_bucket" "docs" {
  name                        = local.docs_bucket_name
  project                     = local.gcp_project
  location                    = local.gcp_region
  uniform_bucket_level_access = true
  versioning {
    enabled = true
  }
}

data "google_iam_policy" "docs" {
  binding {
    role    = "roles/storage.admin"
    members = [for owner in local.additional_owners : "user:${owner}"]
  }

  binding {
    role    = "roles/storage.admin"
    members = ["serviceAccount:${google_service_account.bq_access_sa.email}"]
  }
}

resource "google_storage_bucket_iam_policy" "docs" {
  bucket      = google_storage_bucket.docs.name
  policy_data = data.google_iam_policy.docs.policy_data
}
