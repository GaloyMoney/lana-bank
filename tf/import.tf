locals {
  bq_tables = local.setup_bq ? [
    "user_events",
    "customer_events",
    "loan_events",
    "withdraw_events",
    "deposit_events",
    "credit_facility_events",
    "terms_template_events",
    "interest_accrual_events",
  ] : []
}

import {
  for_each = local.setup_bq ? toset(local.bq_tables) : []
  to       = module.setup.google_bigquery_table.entities[each.value]
  id       = "projects/${local.project_id}/datasets/${local.dataset_id}/tables/${each.value}"
}

import {
  for_each = local.setup_bq ? toset(["sumsub_applicants"]) : []
  to       = module.setup.google_bigquery_table.sumsub_applicants[0]
  id       = "projects/${local.project_id}/datasets/${local.dataset_id}/tables/${each.value}"
}

import {
  for_each = local.setup_bq ? toset(["price_cents_btc"]) : []
  to       = module.setup.google_bigquery_table.price_cents_btc[0]
  id       = "projects/${local.project_id}/datasets/${local.dataset_id}/tables/${each.value}"
}
