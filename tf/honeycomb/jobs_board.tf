data "honeycombio_query_specification" "job_runs" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "job_type"
    op     = "exists"
  }

  breakdowns = ["job_type"]

  time_range = 604800
}

resource "honeycombio_query" "job_runs" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.job_runs.json
}

resource "honeycombio_query_annotation" "job_runs" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.job_runs.id
  name     = "Job runs"
}

data "honeycombio_query_specification" "attempt" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "attempt"
    op     = "exists"
  }

  filter {
    column = "attempt"
    op     = ">"
    value  = "1"
  }

  filter {
    column = "name"
    op     = "contains"
    value  = "retry_wrapper"
  }

  breakdowns = ["attempt", "name"]

  time_range = 604800
}

resource "honeycombio_query" "attempt" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.attempt.json
}

resource "honeycombio_query_annotation" "attempt" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.attempt.id
  name     = "Multiple attempts"
}

data "honeycombio_query_specification" "concurrent_modification_errors" {
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
    value  = "job.process"
  }

  breakdowns = ["trace.trace_id", "root.name"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "concurrent_modification_errors" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.concurrent_modification_errors.json
}

resource "honeycombio_query_annotation" "concurrent_modification_errors" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.concurrent_modification_errors.id
  name     = "Concurrent modification errors in jobs"
}

# Jobs dashboard
resource "honeycombio_flexible_board" "jobs" {
  name        = "${local.name_prefix}-jobs"
  description = "Job execution metrics for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.job_runs.id
      query_annotation_id = honeycombio_query_annotation.job_runs.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.attempt.id
      query_annotation_id = honeycombio_query_annotation.attempt.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.concurrent_modification_errors.id
      query_annotation_id = honeycombio_query_annotation.concurrent_modification_errors.id
      query_style         = "graph"
    }
  }
}

