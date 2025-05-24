with initialized as (
    select
        id as credit_facility_id,
        recorded_at as initialized_recorded_at,

        json_value(event, "$.customer_id") as customer_id,

        cast(json_value(event, '$.amount') as numeric) / {{ var('cents_per_usd') }} as facility_amount_usd,

        cast(json_value(event, "$.terms.annual_rate") as numeric) as annual_rate,
        cast(json_value(event, "$.terms.one_time_fee_rate") as numeric) as one_time_fee_rate,

        cast(json_value(event, "$.terms.initial_cvl") as numeric) as initial_cvl,
        cast(json_value(event, "$.terms.liquidation_cvl") as numeric) as liquidation_cvl,
        cast(json_value(event, "$.terms.margin_call_cvl") as numeric) as margin_call_cvl,

        cast(json_value(event, "$.terms.duration.value") as integer) as duration_value,
        json_value(event, "$.terms.duration.type") as duration_type,

        json_value(event, "$.terms.accrual_interval.type") as accrual_interval,
        json_value(event, "$.terms.accrual_cycle_interval.type") as accrual_cycle_interval,

        cast(json_value(event, "$.terms.interest_due_duration.value") as integer) as interest_due_duration_value,
        json_value(event, "$.terms.interest_due_duration.type") as interest_due_duration_type,

        cast(json_value(event, "$.terms.interest_overdue_duration.value") as integer) as interest_overdue_duration_value,
        json_value(event, "$.terms.interest_overdue_duration.type") as interest_overdue_duration_type,

        json_value(event, "$.account_ids.facility_account_id") as facility_account_id,
        json_value(event, "$.account_ids.collateral_account_id") as collateral_account_id,
        json_value(event, "$.account_ids.fee_income_account_id") as fee_income_account_id,
        json_value(event, "$.account_ids.interest_income_account_id") as interest_income_account_id,
        json_value(event, "$.account_ids.interest_defaulted_account_id") as interest_defaulted_account_id,
        json_value(event, "$.account_ids.disbursed_defaulted_account_id") as disbursed_defaulted_account_id,
        json_value(event, "$.account_ids.interest_receivable_due_account_id") as interest_receivable_due_account_id,
        json_value(event, "$.account_ids.disbursed_receivable_due_account_id") as disbursed_receivable_due_account_id,
        json_value(event, "$.account_ids.interest_receivable_overdue_account_id") as interest_receivable_overdue_account_id,
        json_value(event, "$.account_ids.disbursed_receivable_overdue_account_id") as disbursed_receivable_overdue_account_id,
        json_value(event, "$.account_ids.interest_receivable_not_yet_due_account_id") as interest_receivable_not_yet_due_account_id,
        json_value(event, "$.account_ids.disbursed_receivable_not_yet_due_account_id") as disbursed_receivable_not_yet_due_account_id,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = 'initialized'
)

, approved as (
    select
        id as credit_facility_id,
        recorded_at as approved_recorded_at,
        cast(json_value(event, '$.approved') as boolean) as approved,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = "approval_process_concluded"
)

, activated as (
    select
        id as credit_facility_id,
        recorded_at as activated_recorded_at,
        cast(json_value(event, "$.activated_at") as timestamp) as activated_at,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = "activated"
)

, final as (
    select
        *,
        case when duration_type = 'months' then timestamp_add(date(activated_at), interval duration_value month) end as maturity_at,
    from initialized
    left join approved using (credit_facility_id)
    left join activated using (credit_facility_id)
)


select * from final
