with
    nodes as (
        select
            json_value(event, "$.chart_id") as chart_id,
            {{ target.schema }}.udf_json_array_to_code(
                json_extract(event, "$.spec.code.sections"), ""
            ) as code,
            {{ target.schema }}.udf_json_array_to_code(
                json_extract(event, "$.spec.code.sections"), "."
            ) as dotted_code,
            {{ target.schema }}.udf_json_array_to_code(
                json_extract(event, "$.spec.code.sections"), " "
            ) as spaced_code,
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

select *
from nodes
