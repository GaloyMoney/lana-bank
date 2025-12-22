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
  query_json = data.honeycombio_query_specification.process_type.json
}

resource "honeycombio_query_annotation" "process_type" {
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
  query_json = data.honeycombio_query_specification.events.json
}

resource "honeycombio_query_annotation" "events" {
  query_id = honeycombio_query.events.id
  name     = "Events job queries"
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
  query_json = data.honeycombio_query_specification.handled.json
}

resource "honeycombio_query_annotation" "handled" {
  query_id = honeycombio_query.handled.id
  name     = "handled job queries"
}

# Event Jobs dashboard
resource "honeycombio_flexible_board" "event_jobs" {
  name        = "${local.name_prefix}-event-jobs"
  description = "Jobs and event processing metrics for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.process_type.id
      query_annotation_id = honeycombio_query_annotation.process_type.id
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
}
