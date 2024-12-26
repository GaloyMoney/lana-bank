output "service_account_key_base64" {
  value = google_service_account_key.bq_access_sa_key.private_key
}

output "service_account_email" {
  value = google_service_account.bq_access_sa.email
}

output "dataset_id" {
  value = google_bigquery_dataset.dataset.dataset_id
}

output "gcp_location" {
  value = local.gcp_region
}

output "dataform_repo_name" {
  value = local.dataform_repo_name
}

output "dataform_output_dataset" {
  value = local.dataform_dataset_name
}

output "dataform_release_config" {
  value = local.dataform_release_config_name
}

output "reports_root_folder" {
  value = "reports"
}

output "bucket_name" {
  value = local.docs_bucket_name
}

output "holistics_service_account_key_base64" {
  value = google_service_account_key.holistics_key.private_key
}

output "holistics_service_account_email" {
  value = google_service_account.holistics.email
}
