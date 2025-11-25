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

data "honeycombio_query_specification" "pending_credit_facilities" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "pending_credit_facility_id"
    op     = "exists"
  }

  breakdowns = ["trace.trace_id", "pending_credit_facility_id", "root.name"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "pending_credit_facilities" {
  query_json = data.honeycombio_query_specification.pending_credit_facilities.json
}

resource "honeycombio_query_annotation" "pending_credit_facilities" {
  query_id = honeycombio_query.pending_credit_facilities.id
  name     = "Pending Credit Facility queries"
}

data "honeycombio_query_specification" "credit_facilities" {
  calculation {
    op = "COUNT"
  }

  filter {
    column = "credit_facility_id"
    op     = "exists"
  }

  breakdowns = ["trace.trace_id", "credit_facility_id", "root.name"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  time_range = 604800
}

resource "honeycombio_query" "credit_facilities" {
  query_json = data.honeycombio_query_specification.credit_facilities.json
}

resource "honeycombio_query_annotation" "credit_facilities" {
  query_id = honeycombio_query.credit_facilities.id
  name     = "Credit Facility queries"
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

  panel {
    type = "query"
    query_panel {
      query_id            = honeycombio_query.pending_credit_facilities.id
      query_annotation_id = honeycombio_query_annotation.pending_credit_facilities.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"
    query_panel {
      query_id            = honeycombio_query.credit_facilities.id
      query_annotation_id = honeycombio_query_annotation.credit_facilities.id
      query_style         = "graph"
    }
  }
}
