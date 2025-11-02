data "honeycombio_query_specification" "retry_macro" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "name"
    op     = "contains"
    value  = "wrapper"
  }

  breakdowns = ["name", "attempt"]

  time_range = 604800
}

resource "honeycombio_query" "retry_macro" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.retry_macro.json
}

resource "honeycombio_query_annotation" "retry_macro" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.retry_macro.id
  name     = "Retry attempts"
}

# Jobs dashboard
resource "honeycombio_flexible_board" "retry_macro" {
  name        = "${local.name_prefix}-retry-macro"
  description = "Retry attempts for for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.retry_macro.id
      query_annotation_id = honeycombio_query_annotation.retry_macro.id
      query_style         = "graph"
    }
  }
}

