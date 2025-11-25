
data "honeycombio_query_specification" "price_current" {

    calculation {
        op     = "MAX"
        column = "price"
    }

    filter {
        column = "name"
        op     = "="
        value  = "core.price.listen_for_updates.process_message"
    }

    time_range = 86400
}

resource "honeycombio_query" "price_current" {
    query_json = data.honeycombio_query_specification.price_current.json
}

resource "honeycombio_query_annotation" "price_current" {
    query_id = honeycombio_query.price_current.id
    name     = "Current BTC price"
}

resource "honeycombio_flexible_board" "price_board" {

    name = "${local.name_prefix}-price"
    description = "Price metrics for ${local.name_prefix}"

    panel {
        type = "query"
        query_panel {
            query_id            = honeycombio_query.price_current.id
            query_annotation_id = honeycombio_query_annotation.price_current.id
            query_style         = "graph"
        }
    }
}
