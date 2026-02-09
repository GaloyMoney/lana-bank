with
    approved_credit_facilities as (
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
        select
            credit_facility_id,
            disbursal_created_at as disbursal_initialized_recorded_at,
            effective as disbursal_settled_recorded_at,
            amount_usd as total_disbursed_usd,
            disbursal_id,
            obligation_id,
            min(effective) over (
                partition by credit_facility_id
            ) as min_disbursal_settled_recorded_at,
            amount_usd
            / sum(amount_usd) over (partition by credit_facility_id) as disbursal_ratio
        from {{ ref("int_core_disbursal_events_rollup") }}
        where is_settled
    ),

    interest as (
        select
            credit_facility_id,
            sum(
                coalesce(posted_total_interest_usd, 0)
            ) as cf_total_interest_incurred_usd
        from {{ ref("int_core_interest_accrual_cycle_events_rollup") }}
        group by credit_facility_id
    ),

    payments as (
        select
            credit_facility_id,
            sum(coalesce(interest_usd, 0)) as cf_total_interest_paid_usd,
            sum(coalesce(disbursal_usd, 0)) as cf_total_disbursal_paid_usd,
            max(
                if(interest_usd > 0, effective, null)
            ) as most_recent_interest_payment_timestamp,
            max(
                if(disbursal_usd > 0, effective, null)
            ) as most_recent_disbursal_payment_timestamp
        from {{ ref("int_payment_events") }}
        group by credit_facility_id
    ),

    interest_paid_stats as (
        select
            credit_facility_id,
            disbursal_id,
            disbursal_ratio,
            cf_total_interest_incurred_usd,
            cf_total_interest_paid_usd,
            cf_total_disbursal_paid_usd,
            timestamp_diff(
                most_recent_interest_payment_timestamp,
                disbursal_settled_recorded_at,
                day
            ) as disbursal_interest_days,
            timestamp_diff(
                most_recent_interest_payment_timestamp,
                min_disbursal_settled_recorded_at,
                day
            ) as credit_facility_interest_days
        from disbursals
        left join payments using (credit_facility_id)
        left join interest using (credit_facility_id)
    ),

    interest_paid as (
        select
            credit_facility_id,
            disbursal_id,
            disbursal_interest_days,
            credit_facility_interest_days,
            disbursal_ratio
            * disbursal_interest_days as disbursal_weighted_interest_days,
            safe_divide(
                disbursal_ratio * disbursal_interest_days,
                sum(disbursal_ratio * disbursal_interest_days) over (
                    partition by credit_facility_id
                )
            ) as interest_paid_ratio,
            cf_total_interest_paid_usd * safe_divide(
                disbursal_ratio * disbursal_interest_days,
                sum(disbursal_ratio * disbursal_interest_days) over (
                    partition by credit_facility_id
                )
            ) as interest_paid_usd,
            cf_total_interest_incurred_usd * safe_divide(
                disbursal_ratio * disbursal_interest_days,
                sum(disbursal_ratio * disbursal_interest_days) over (
                    partition by credit_facility_id
                )
            ) as interest_incurred_usd,
            disbursal_ratio * cf_total_disbursal_paid_usd as disbursal_paid_usd
        from interest_paid_stats
    ),

    final as (
        select
            credit_facility_id,
            disbursal_id,
            credit_facility_created_at as facility_initialized_recorded_at,
            credit_facility_activated_at as facility_approved_recorded_at,
            credit_facility_activated_at as facility_activated_recorded_at,
            credit_facility_maturity_at as facility_maturity_at,
            credit_facility_activated_at as facility_start_date,
            credit_facility_maturity_at as facility_end_date,
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
            disbursal_initialized_recorded_at,

            disbursal_settled_recorded_at,
            disbursal_settled_recorded_at as disbursal_approved_recorded_at,
            disbursal_settled_recorded_at as disbursal_start_date,
            credit_facility_maturity_at as disbursal_end_date,
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
            obligation_id,

            coalesce(facility_amount_usd, 0) as facility_amount_usd,

            coalesce(
                cf_total_interest_incurred_usd, 0
            ) as cf_total_interest_incurred_usd,
            coalesce(cf_total_interest_paid_usd, 0) as cf_total_interest_paid_usd,
            coalesce(cf_total_disbursal_paid_usd, 0) as cf_total_disbursal_paid_usd,
            coalesce(collateral_amount_usd, 0) as cf_total_collateral_amount_usd,
            coalesce(
                disbursal_ratio * collateral_amount_usd, 0
            ) as collateral_amount_usd,
            coalesce(total_disbursed_usd, 0) as total_disbursed_usd,
            coalesce(disbursal_interest_days, 0) as disbursal_interest_days,
            coalesce(credit_facility_interest_days, 0) as credit_facility_interest_days,
            coalesce(
                disbursal_weighted_interest_days, 0
            ) as disbursal_weighted_interest_days,
            coalesce(interest_incurred_usd, 0) as interest_incurred_usd,
            coalesce(interest_paid_ratio, 0) as interest_paid_ratio,
            coalesce(interest_paid_usd, 0) as interest_paid_usd,
            coalesce(disbursal_paid_usd, 0) as disbursal_paid_usd,
            credit_facility_maturity_at < current_date() as matured

        from approved_credit_facilities
        inner join disbursals using (credit_facility_id)
        left join collateral_deposits using (credit_facility_id)
        left join interest using (credit_facility_id)
        left join payments using (credit_facility_id)
        left join interest_paid using (credit_facility_id, disbursal_id)
    )

select *
from final
