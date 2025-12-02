select
    id as account_set_id,
    set_name,
    row_number() over () as set_key

from {{ ref('stg_account_sets') }}
where loaded_to_dw_at >= (
    select coalesce(max(loaded_to_dw_at), '1900-01-01')
    from {{ ref('stg_core_chart_node_events') }}
    where event_type = 'initialized'
)
