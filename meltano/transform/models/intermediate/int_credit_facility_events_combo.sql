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
        cf.*,
        c.* except (credit_facility_id),
        collateral_amount_usd / facility_amount_usd * 100 as current_facility_cvl,
        ac.* except (credit_facility_id),
    from credit_facility as cf
    left join collateral as c using (credit_facility_id)
    left join accrual_cycle as ac using (credit_facility_id)
)


select * from final
