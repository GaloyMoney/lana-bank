with credit_facility as (
    select
        *
    from {{ ref('int_credit_facility_events') }}
)

, collateral as (
    select
        *
    from {{ ref('int_credit_facility_collateral_events') }}
)

, accrual_cycle as (
    select
        *
    from {{ ref('int_credit_facility_accrual_cycle_events') }}
)

, final as (
    select
        *
    from credit_facility
    left join collateral using (credit_facility_id)
    left join accrual_cycle using (credit_facility_id)
)


select * from final
