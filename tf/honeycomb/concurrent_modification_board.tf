data "honeycombio_query_specification" "concurrent_modifications_jobs_frequency" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "error"
    op     = "="
    value  = "true"
  }

  filter {
    column = "exception.message"
    op     = "contains"
    value  = "ConcurrentModification"
  }

  filter {
    column = "root.name"
    op     = "contains"
    value  = "job"
  }

  breakdowns = ["trace.trace_id", "root.name"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "concurrent_modifications_jobs_frequency" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.concurrent_modifications_jobs_frequency.json
}

resource "honeycombio_query_annotation" "concurrent_modifications_jobs_frequency" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.concurrent_modifications_jobs_frequency.id
  name     = "Jobs with concurrent modification errors"
}

data "honeycombio_query_specification" "concurrent_modifications_jobs_and_events_frequency" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "root.name"
    op     = "contains"
    value  = "job"
  }

  filter {
    column = "event_type"
    op     = "exists"
  }

  filter {
    column = "root.name"
    op     = "does-not-contain"
    value  = "outbox.core"
  }

  filter {
    column = "root.name"
    op     = "does-not-contain"
    value  = "ephemeral"
  }

  breakdowns = ["event_type", "trace.trace_id", "root.name"]

  time_range = 604800
}

resource "honeycombio_query" "concurrent_modifications_jobs_and_events_frequency" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.concurrent_modifications_jobs_and_events_frequency.json
}

resource "honeycombio_query_annotation" "concurrent_modifications_jobs_and_events_frequency" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.concurrent_modifications_jobs_and_events_frequency.id
  name     = "Jobs and their triggering events"
}

resource "honeycombio_flexible_board" "concurrent_modifications" {
  name        = "${local.name_prefix}-concurrent-modifications"
  description = "Track concurrent modification errors across jobs and event processing in ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.concurrent_modifications_jobs_frequency.id
      query_annotation_id = honeycombio_query_annotation.concurrent_modifications_jobs_frequency.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.concurrent_modifications_jobs_and_events_frequency.id
      query_annotation_id = honeycombio_query_annotation.concurrent_modifications_jobs_and_events_frequency.id
      query_style         = "graph"
    }
  }
}
