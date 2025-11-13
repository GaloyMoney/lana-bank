data "honeycombio_query_specification" "nginx_requests_by_host" {
  calculation {
    op = "COUNT"
  }

  calculation {
    op     = "P50"
    column = "duration_ms"
  }

  filter {
    column = "http.host"
    op     = "exists"
  }

  breakdowns = ["http.host"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  limit      = 100
  time_range = 604800
}

resource "honeycombio_query" "nginx_requests_by_host" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.nginx_requests_by_host.json
}

resource "honeycombio_query_annotation" "nginx_requests_by_host" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.nginx_requests_by_host.id
  name     = "Requests by host"
}

data "honeycombio_query_specification" "nginx_root_requests_by_ip" {
  calculation {
    op = "COUNT"
  }

  calculation {
    op     = "P50"
    column = "duration_ms"
  }

  filter {
    column = "service.name"
    op     = "="
    value  = "${local.name_prefix}-ingress"
  }

  filter {
    column = "is_root"
    op     = "="
    value  = "true"
  }

  breakdowns = ["net.peer.ip", "http.user_agent"]

  order {
    op    = "COUNT"
    order = "descending"
  }

  limit      = 100
  time_range = 604800
}

resource "honeycombio_query" "nginx_root_requests_by_ip" {
  dataset    = var.honeycomb_dataset
  query_json = data.honeycombio_query_specification.nginx_root_requests_by_ip.json
}

resource "honeycombio_query_annotation" "nginx_root_requests_by_ip" {
  dataset  = var.honeycomb_dataset
  query_id = honeycombio_query.nginx_root_requests_by_ip.id
  name     = "Root requests by IP and user agent"
}

# Nginx board
resource "honeycombio_flexible_board" "nginx" {
  name        = "${local.name_prefix}-nginx"
  description = "Nginx metrics for ${local.name_prefix}"

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.nginx_requests_by_host.id
      query_annotation_id = honeycombio_query_annotation.nginx_requests_by_host.id
      query_style         = "graph"
    }
  }

  panel {
    type = "query"

    query_panel {
      query_id            = honeycombio_query.nginx_root_requests_by_ip.id
      query_annotation_id = honeycombio_query_annotation.nginx_root_requests_by_ip.id
      query_style         = "table"
    }
  }
}

