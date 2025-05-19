with initialized as (
    select
        id as collateral_id,
        recorded_at as initialized_recorded_at,

        json_value(event, "$.credit_facility_id") as credit_facility_id,
        json_value(event, "$.account_id") as account_id,

    from {{ ref('stg_collateral_events') }}
    where event_type = 'initialized'
)

, updated as (
    select
        id as collateral_id,
        recorded_at as updated_recorded_at,

        json_value(event, "$.action") as action,

        cast(json_value(event, "$.abs_diff") as numeric) as abs_diff,
        cast(json_value(event, "$.new_value") as numeric) as new_value,

        json_value(event, "$.ledger_tx_id") as ledger_tx_id,

    from {{ ref('stg_collateral_events') }}
    where event_type = 'updated'
)

, final as (
    select
        *
    from initialized
    left join updated using (collateral_id)
)


select * from final
