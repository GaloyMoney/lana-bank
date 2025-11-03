# Note: Debug mode needs to be set for these values to be shown
data "honeycombio_query_specification" "logger_db_perf" {
  calculation {
    op     = "MAX"
    column = "elapsed_secs"
  }

  filter {
    column = "name"
    op     = "contains"
    value  = "src/logger.rs"
  }

  filter {
    column = "db.statement"
    op     = "exists"
  }

  filter {
    column = "db.statement"
    op     = "!="
    value  = ""
  }

  breakdowns = ["db.statement"]

  order {
    op     = "MAX"
    column = "elapsed_secs"
    order  = "descending"
  }

  limit      = 100
  time_range = 604800
}

resource "honeycombio_query" "logger_db_perf" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.logger_db_perf.json
}

resource "honeycombio_query_annotation" "logger_db_perf" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.logger_db_perf.id
  name     = "Logger DB performance"
}

# Logger DB dashboard
resource "honeycombio_flexible_board" "logger_db" {
  name        = "${local.name_prefix}-logger-db"
  description = "Logger DB performance metrics for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.logger_db_perf.id
      query_annotation_id = honeycombio_query_annotation.logger_db_perf.id
      query_style         = "graph"
    }
  }
}
