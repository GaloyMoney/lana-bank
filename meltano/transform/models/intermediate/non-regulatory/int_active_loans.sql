with

active_cf as(
    select
        * except (
            interest_accrual_cycle_idx,
            interest_period_start_at,
            interest_period_end_at,
            interest_period_interval_type,
            interest_due_duration_from_accrual_value,
            interest_due_duration_from_accrual_type,
            obligation_overdue_duration_from_due_value,
            obligation_overdue_duration_from_due_type,
            obligation_liquidation_duration_from_due_value,
            obligation_liquidation_duration_from_due_type,
            approval_process_id,
            collateral_id,
            disbursal_credit_account_id,
            interest_accrual_ids,
            audit_entry_ids,
            obligation_ids
        )
    from {{ ref('int_core_credit_facility_events_rollup') }}
    where is_activated and not is_completed
)

, disbursals as(
    select
        credit_facility_id,
        sum(amount_usd) as total_disbursed_usd,
        count(*) as number_disbursals,
    from {{ ref('int_core_disbursal_events_rollup') }}
    group by credit_facility_id
)

, final as (
    select
        active_cf.credit_facility_id,
        active_cf.customer_id,
        active_cf.activated_at,
        active_cf.facility_amount_usd,
        total_disbursed_usd,
        number_disbursals,
        active_cf.* except (
            credit_facility_id,
            customer_id,
            facility_amount_usd,
            activated_at
        )
    from active_cf
    left join disbursals using(credit_facility_id)

)

select *
from final
