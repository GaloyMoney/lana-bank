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
}

