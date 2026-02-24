data "honeycombio_query_specification" "command_job_failures" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "name"
    op     = "contains"
    value  = "job.process_command"
  }

  filter {
    column = "error"
    op     = "="
    value  = "true"
  }

  breakdowns = ["name", "exception.message"]

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
  name     = "Command job failures"
}

data "honeycombio_query_specification" "command_job_error_severity" {
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

  breakdowns = ["error.level", "job_type"]

  time_range = 604800
}

resource "honeycombio_query" "command_job_error_severity" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.command_job_error_severity.json
}

resource "honeycombio_query_annotation" "command_job_error_severity" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.command_job_error_severity.id
  name     = "Command job error severity"
}

data "honeycombio_query_specification" "command_job_retries" {
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
    column = "attempt"
    op     = ">"
    value  = "1"
  }

  breakdowns = ["job_type", "attempt"]

  time_range = 604800
}

resource "honeycombio_query" "command_job_retries" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.command_job_retries.json
}

resource "honeycombio_query_annotation" "command_job_retries" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.command_job_retries.id
  name     = "Command job retries"
}

# Command jobs dashboard
resource "honeycombio_flexible_board" "command_jobs" {
  name        = "${local.name_prefix}-command-jobs"
  description = "Command job health and monitoring for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_job_failures.id
      query_annotation_id = honeycombio_query_annotation.command_job_failures.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_job_error_severity.id
      query_annotation_id = honeycombio_query_annotation.command_job_error_severity.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.command_job_retries.id
      query_annotation_id = honeycombio_query_annotation.command_job_retries.id
      query_style         = "graph"
    }
  }
}
