data "honeycombio_query_specification" "credit_facility_proposals" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "credit_facility_proposal_id"
    op     = "exists"
  }

  breakdowns = ["trace.trace_id", "credit_facility_proposal_id", "root.name"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "credit_facility_proposals" {
  query_json = data.honeycombio_query_specification.credit_facility_proposals.json
}

resource "honeycombio_query_annotation" "credit_facility_proposals" {
  query_id = honeycombio_query.credit_facility_proposals.id
  name     = "Credit Facility Proposals queries"
}

# Credit Board
resource "honeycombio_flexible_board" "credit_board" {
  name        = "${local.name_prefix}-credit"
  description = "Credit metrics for ${local.name_prefix}"

  panel {
    type = "query"
    query_panel {
      query_id            = honeycombio_query.credit_facility_proposals.id
      query_annotation_id = honeycombio_query_annotation.credit_facility_proposals.id
      query_style         = "graph"
    }
  }
}
