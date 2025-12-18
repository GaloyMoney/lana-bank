with

active_cf as (
    select
        * except (
            interest_accrual_cycle_idx,
            approval_process_id,
            collateral_id,
            disbursal_credit_account_id,
            interest_accrual_ids
        )
    from {{ ref('int_core_credit_facility_events_rollup') }}
    where not is_matured
),

disbursals as (
    select
        credit_facility_id,
        sum(amount_usd) as total_disbursed_usd,
        count(*) as number_disbursals
    from {{ ref('int_core_disbursal_events_rollup') }}
    group by credit_facility_id
),

final as (
    select
        active_cf.* except (
            credit_facility_id,
            customer_id,
            facility_amount_usd,
            credit_facility_activated_at
        ),
        active_cf.credit_facility_id,
        active_cf.customer_id,
        active_cf.credit_facility_activated_at,
        active_cf.facility_amount_usd,
        total_disbursed_usd,
        number_disbursals
    from active_cf
    left join disbursals using (credit_facility_id)

)

select *
from final
