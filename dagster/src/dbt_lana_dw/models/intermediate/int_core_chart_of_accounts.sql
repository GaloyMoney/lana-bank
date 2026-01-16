with
    nodes as (
        select
            json_value(event, "$.chart_id") as chart_id,
            array(select string(code) from unnest (json_query_array(json_query((event), "lax $.spec.code.sections.code"), "$")) as code) as code_array,
            json_value(event, "$.spec.name.name") as node_name,
            json_value(event, "$.ledger_account_set_id") as account_set_id
        from {{ ref("stg_core_chart_node_events") }}
        where
            loaded_to_dw_at >= (
                select coalesce(max(loaded_to_dw_at), "1900-01-01")
                from {{ ref("stg_core_chart_node_events") }}
                where event_type = "initialized"
            )
            and event_type = "initialized"
    )

select
    chart_id,
    array_to_string(code_array, "") as code,
    array_to_string(code_array, ".") as dotted_code,
    array_to_string(code_array, " ") as spaced_code,
    node_name,
    account_set_id
from nodes
