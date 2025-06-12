select
    id as account_set_id,
    set_name,
    row_number() over () as set_key

from {{ ref('stg_account_sets') }}
where _sdc_batched_at >= (
    select coalesce(max(_sdc_batched_at), '1900-01-01')
    from {{ ref('stg_core_chart_events') }}
    where event_type = 'initialized'
)
