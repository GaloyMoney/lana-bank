select
    cast(
        json_value(
            any_value(values having max recorded_at), "$.settled.cr_balance"
        ) as numeric
    ) as settled_cr,
    cast(
        json_value(
            any_value(values having max recorded_at), "$.settled.dr_balance"
        ) as numeric
    ) as settled_dr,
    json_value(values, "$.account_id") as account_id,
    json_value(values, "$.currency") as currency

from {{ ref('stg_account_balances') }}

where _sdc_batched_at >= (
    select coalesce(max(_sdc_batched_at), '1900-01-01')
    from {{ ref('stg_core_chart_events') }}
    where event_type = 'initialized'
)

group by account_id, currency
