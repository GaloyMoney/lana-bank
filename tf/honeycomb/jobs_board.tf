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
  name     = "Concurrent modification retries"
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

data "honeycombio_query_specification" "process_type" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "event_type"
    op     = "exists"
  }

  filter {
    column = "process_type"
    op     = "exists"
  }

  breakdowns = ["seq", "event_type", "name", "process_type"]

  order {
    column = "seq"
    order  = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "process_type" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.process_type.json
}

resource "honeycombio_query_annotation" "process_type" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.process_type.id
  name     = "Governance approval queries"
}

data "honeycombio_query_specification" "events" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "event_type"
    op     = "exists"
  }

  breakdowns = ["seq", "event_type", "name"]

  order {
    column = "seq"
    order  = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "events" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.events.json
}

resource "honeycombio_query_annotation" "events" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.events.id
  name     = "Event types processed"
}

data "honeycombio_query_specification" "handled" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "handled"
    op     = "exists"
  }

  breakdowns = ["seq", "handled"]

  order {
    column = "seq"
    order  = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "handled" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.handled.json
}

resource "honeycombio_query_annotation" "handled" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.handled.id
  name     = "Event handler hit rate"
}

data "honeycombio_query_specification" "handler_duration" {
  calculation {
    op     = "P50"
    column = "duration_ms"
  }
  calculation {
    op     = "P95"
    column = "duration_ms"
  }

  filter {
    column = "name"
    op     = "contains"
    value  = "job.process_message"
  }

  breakdowns = ["name"]

  order {
    op     = "P95"
    column = "duration_ms"
    order  = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "handler_duration" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.handler_duration.json
}

resource "honeycombio_query_annotation" "handler_duration" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.handler_duration.id
  name     = "Event handler duration"
}

data "honeycombio_query_specification" "command_job_duration" {
  calculation {
    op     = "P50"
    column = "duration_ms"
  }
  calculation {
    op     = "P95"
    column = "duration_ms"
  }

  filter {
    column = "name"
    op     = "contains"
    value  = "job.process_command"
  }

  breakdowns = ["name"]

  order {
    op     = "P95"
    column = "duration_ms"
    order  = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "command_job_duration" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.command_job_duration.json
}

resource "honeycombio_query_annotation" "command_job_duration" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.command_job_duration.id
  name     = "Command job duration"
}

data "honeycombio_query_specification" "command_job_conclusions" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "name"
    op     = "="
    value  = "job.execute_job"
  }

  filter {
    column = "job_type"
    op     = "contains"
    value  = "command."
  }

  breakdowns = ["conclusion", "job_type"]

  time_range = 604800
}

resource "honeycombio_query" "command_job_conclusions" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.command_job_conclusions.json
}

resource "honeycombio_query_annotation" "command_job_conclusions" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.command_job_conclusions.id
  name     = "Command job conclusions"
}

data "honeycombio_query_specification" "handler_trace_errors" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "name"
    op     = "="
    value  = "job.execute_job"
  }

  filter {
    column = "job_type"
    op     = "contains"
    value  = "outbox."
  }

  filter {
    column = "error"
    op     = "="
    value  = "true"
  }

  breakdowns = ["job_type", "error.message", "attempt"]

  time_range = 604800
}

resource "honeycombio_query" "handler_trace_errors" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.handler_trace_errors.json
}

resource "honeycombio_query_annotation" "handler_trace_errors" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.handler_trace_errors.id
  name     = "Event handler errors and retries"
}

data "honeycombio_query_specification" "command_job_failures" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "name"
    op     = "="
    value  = "job.execute_job"
  }

  filter {
    column = "job_type"
    op     = "contains"
    value  = "command."
  }

  filter {
    column = "error"
    op     = "="
    value  = "true"
  }

  breakdowns = ["job_type", "error.message", "attempt"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "command_job_failures" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.command_job_failures.json
}

resource "honeycombio_query_annotation" "command_job_failures" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.command_job_failures.id
  name     = "Command job errors and retries"
}

data "honeycombio_query_specification" "command_types" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "name"
    op     = "="
    value  = "job.execute_job"
  }

  filter {
    column = "job_type"
    op     = "contains"
    value  = "command."
  }

  breakdowns = ["job_type"]

  time_range = 604800
}

resource "honeycombio_query" "command_types" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.command_types.json
}

resource "honeycombio_query_annotation" "command_types" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.command_types.id
  name     = "Command job types processed"
}

# Jobs dashboard
resource "honeycombio_flexible_board" "jobs" {
  name        = "${local.name_prefix}-jobs"
  description = "Job execution and event processing metrics for ${local.name_prefix}"

  # Row 1: Overview
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
      query_id            = honeycombio_query.concurrent_modification_errors.id
      query_annotation_id = honeycombio_query_annotation.concurrent_modification_errors.id
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

  # Row 2-3: Duration (both panels are double-height due to P50+P95, placed side by side)
  # Event handler panels fill col 3 of both rows
  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.handler_duration.id
      query_annotation_id = honeycombio_query_annotation.handler_duration.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_job_duration.id
      query_annotation_id = honeycombio_query_annotation.command_job_duration.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.events.id
      query_annotation_id = honeycombio_query_annotation.events.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.handled.id
      query_annotation_id = honeycombio_query_annotation.handled.id
      query_style         = "graph"
    }
  }

  # Row 4: Event handlers
  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.handler_trace_errors.id
      query_annotation_id = honeycombio_query_annotation.handler_trace_errors.id
      query_style         = "graph"
    }
  }

  # Row 4: Command jobs
  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_types.id
      query_annotation_id = honeycombio_query_annotation.command_types.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_job_conclusions.id
      query_annotation_id = honeycombio_query_annotation.command_job_conclusions.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_job_failures.id
      query_annotation_id = honeycombio_query_annotation.command_job_failures.id
      query_style         = "graph"
    }
  }

  # Row 5: Specialized
  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.process_type.id
      query_annotation_id = honeycombio_query_annotation.process_type.id
      query_style         = "graph"
    }
  }
}
