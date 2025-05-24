with collateral_state_changed as (
    select
        id as credit_facility_id,
        max(recorded_at) as collateral_state_changed_recorded_at,
        cast(json_value(any_value(event having max recorded_at), '$.collateral') as numeric) as collateral,
        cast(json_value(any_value(event having max recorded_at), '$.price') as numeric) as price,
        json_value(any_value(event having max recorded_at), '$.state') as state,
        cast(json_value(any_value(event having max recorded_at), '$.outstanding.interest') as numeric) as outstanding_interest,
        cast(json_value(any_value(event having max recorded_at), '$.outstanding.disbursed') as numeric) as outstanding_disbursed,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = 'collateralization_state_changed'
    group by credit_facility_id
),

final as (
    select
        credit_facility_id,
        collateral_state_changed_recorded_at,
        collateral as collateral_amount_sats,
        collateral / {{ var('sats_per_bitcoin') }} as collateral_amount_btc,
        price / {{ var('cents_per_usd') }} as price_usd_per_btc,
        collateral / {{ var('sats_per_bitcoin') }} * price / {{ var('cents_per_usd') }} as collateral_amount_usd,
        state,
        outstanding_interest,
        outstanding_disbursed,

    from collateral_state_changed

)

select * from final
