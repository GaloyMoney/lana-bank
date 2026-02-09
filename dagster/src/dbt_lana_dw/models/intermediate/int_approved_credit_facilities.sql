with
    approved as (
        select * from {{ ref("int_core_credit_facility_events_rollup") }} where approved
    ),

    collateral_deposits as (
        select
            credit_facility_id,
            max(credit_facility_modified_at) as most_recent_collateral_deposit_at,
            any_value(
                collateral_amount_btc having max credit_facility_modified_at
            ) as most_recent_collateral_deposit_amount_btc
        from {{ ref("int_core_credit_facility_events_rollup_sequence") }}
        group by credit_facility_id
    ),

    disbursals as (
        select credit_facility_id, sum(coalesce(amount_usd, 0)) as total_disbursed_usd
        from {{ ref("int_core_disbursal_events_rollup") }}
        where collateralization_state_changed
        group by credit_facility_id
    ),

    interest as (
        select
            credit_facility_id,
            sum(coalesce(posted_total_interest_usd, 0)) as total_interest_incurred_usd
        from {{ ref("int_core_interest_accrual_cycle_events_rollup") }}
        group by credit_facility_id
    ),

    payments as (
        select
            credit_facility_id,
            sum(coalesce(interest_usd, 0)) as total_interest_paid_usd,
            sum(coalesce(disbursal_usd, 0)) as total_disbursal_paid_usd,
            max(
                if(interest_usd > 0, effective, null)
            ) as most_recent_interest_payment_timestamp,
            max(
                if(disbursal_usd > 0, effective, null)
            ) as most_recent_disbursal_payment_timestamp
        from {{ ref("int_payment_events") }}
        group by credit_facility_id
    ),

    final as (
        select
            credit_facility_id,
            credit_facility_created_at,
            credit_facility_activated_at,
            credit_facility_maturity_at,
            credit_facility_activated_at as start_date,
            credit_facility_maturity_at as end_date,
            annual_rate,
            one_time_fee_rate,
            initial_cvl,
            liquidation_cvl,
            margin_call_cvl,
            duration_value,
            duration_type,
            accrual_interval,
            accrual_cycle_interval,
            most_recent_interest_payment_timestamp,
            most_recent_disbursal_payment_timestamp,
            most_recent_collateral_deposit_at,

            customer_id,
            facility_account_id,
            collateral_account_id,
            fee_income_account_id,
            interest_income_account_id,
            interest_defaulted_account_id,
            disbursed_defaulted_account_id,

            interest_receivable_due_account_id,
            disbursed_receivable_due_account_id,
            interest_receivable_overdue_account_id,
            disbursed_receivable_overdue_account_id,
            interest_receivable_not_yet_due_account_id,
            disbursed_receivable_not_yet_due_account_id,
            coalesce(facility_amount_usd, 0) as facility_amount_usd,
            coalesce(total_interest_paid_usd, 0) as total_interest_paid_usd,
            coalesce(total_disbursal_paid_usd, 0) as total_disbursal_paid_usd,
            coalesce(total_interest_incurred_usd, 0) as total_interest_incurred_usd,
            coalesce(collateral_amount_usd, 0) as total_collateral_amount_usd,
            coalesce(total_disbursed_usd, 0) as total_disbursed_usd,
            credit_facility_maturity_at < current_date() as matured,
            row_number() over () as credit_facility_key

        from approved
        left join collateral_deposits using (credit_facility_id)
        left join disbursals using (credit_facility_id)
        left join interest using (credit_facility_id)
        left join payments using (credit_facility_id)
    )

select *
from final
