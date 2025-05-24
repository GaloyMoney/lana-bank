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

        cast(json_value(event, "$.abs_diff") as numeric) as collateral_abs_diff_amount_sats,
        cast(json_value(event, "$.new_value") as numeric) as collateral_new_amount_sats,

        json_value(event, "$.ledger_tx_id") as ledger_tx_id,

    from {{ ref('stg_collateral_events') }}
    where event_type = 'updated'
)

, final as (
    select
        i.*,
        u.updated_recorded_at,
        u.action,
        collateral_abs_diff_amount_sats / {{ var('sats_per_bitcoin') }} as collateral_abs_diff_amount_btc,
        collateral_new_amount_sats / {{ var('sats_per_bitcoin') }} as collateral_new_amount_btc,
        u.ledger_tx_id,
    from initialized as i
    left join updated as u using (collateral_id)
)


select * from final
