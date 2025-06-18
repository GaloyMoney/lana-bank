with nodes as(
    select
        {{ target.schema }}.udf_json_array_to_code(json_extract(event, "$.spec.code.sections"), '') as code,
        {{ target.schema }}.udf_json_array_to_code(json_extract(event, "$.spec.code.sections"), '.') as dotted_code,
        {{ target.schema }}.udf_json_array_to_code(json_extract(event, "$.spec.code.sections"), ' ') as spaced_code,
        json_value(event, "$.spec.name.name") as name,
        json_value(event, "$.ledger_account_set_id") as account_set_id,
    from {{ ref('stg_core_chart_events') }}
    where _sdc_batched_at >= (
        select coalesce(max(_sdc_batched_at), '1900-01-01')
        from {{ ref('stg_core_chart_events') }}
        where event_type = 'initialized'
    )
    and event_type = 'node_added'
)

select * from nodes
