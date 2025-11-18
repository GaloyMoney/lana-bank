data "honeycombio_query_specification" "graphql_operations" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "graphql.operation_name"
    op     = "exists"
  }

  breakdowns = ["graphql.operation_type", "graphql.operation_name", "graphql.query"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  limit      = 100
  time_range = 604800
}

resource "honeycombio_query" "graphql_operations" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.graphql_operations.json
}

resource "honeycombio_query_annotation" "graphql_operations" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.graphql_operations.id
  name     = "GraphQL operations"
}

data "honeycombio_query_specification" "graphql_slowest" {
  calculation {
    op     = "P50"
    column = "duration_ms"
  }

  calculation {
    op     = "P90"
    column = "duration_ms"
  }

  filter {
    column = "graphql.operation_name"
    op     = "exists"
  }

  breakdowns = ["graphql.operation_name", "graphql.operation_type"]

  order {
    op     = "P90"
    column = "duration_ms"
    order  = "descending"
  }

  limit      = 100
  time_range = 604800
}

resource "honeycombio_query" "graphql_slowest" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.graphql_slowest.json
}

resource "honeycombio_query_annotation" "graphql_slowest" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.graphql_slowest.id
  name     = "Slowest GraphQL operations"
}

# GraphQL operations dashboard
resource "honeycombio_flexible_board" "graphql" {
  name        = "${local.name_prefix}-graphql"
  description = "GraphQL operations for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.graphql_operations.id
      query_annotation_id = honeycombio_query_annotation.graphql_operations.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.graphql_slowest.id
      query_annotation_id = honeycombio_query_annotation.graphql_slowest.id
      query_style         = "graph"
    }
  }
}

