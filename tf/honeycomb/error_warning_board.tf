data "honeycombio_query_specification" "error_warnings" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "error"
    op     = "="
    value  = "true"
  }

  filter {
    column = "span.kind"
    op     = "does-not-exist"
  }

  breakdowns = ["error", "level", "exception.message"]

  time_range = 604800
}

resource "honeycombio_query" "error_warnings" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.error_warnings.json
}

resource "honeycombio_query_annotation" "error_warnings" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.error_warnings.id
  name     = "Errors and Warnings"
}

# Errors and warnings board
resource "honeycombio_flexible_board" "error_warnings" {
  name        = "${local.name_prefix}-errors-warnings"
  description = "Errors and warnings for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.error_warnings.id
      query_annotation_id = honeycombio_query_annotation.error_warnings.id
      query_style         = "graph"
    }
  }
}

