with collateral as (
    select
        id as collateral_id,
        credit_facility_id,

        action,
        cast(abs_diff as numeric) as abs_diff_sats,
        cast(abs_diff as numeric) / {{ var('sats_per_bitcoin') }} as abs_diff_btc,
        cast(collateral_amount as numeric) as collateral_amount_sats,
        cast(collateral_amount as numeric) / {{ var('sats_per_bitcoin') }} as collateral_amount_btc,
        account_id,
        created_at as collateral_created_at,
        modified_at as collateral_modified_at,

        * except(
            id,
            credit_facility_id,
            action,
            abs_diff,
            collateral_amount,
            account_id,
            created_at,
            modified_at,

            last_sequence,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_collateral_events_rollup') }}
)


select * from collateral
