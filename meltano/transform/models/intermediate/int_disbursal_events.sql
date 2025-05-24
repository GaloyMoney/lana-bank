with initialized as (
    select
        id as disbursal_id,
        recorded_at as initialized_recorded_at,
        json_value(event, '$.facility_id') as credit_facility_id,
        cast(json_value(event, '$.amount') as numeric) / {{ var('cents_per_usd') }} as disbursal_amount_usd,

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

    from {{ ref('stg_disbursal_events') }}
    where event_type = 'initialized'
)

, concluded as (
    select
        id as disbursal_id,
        recorded_at as concluded_recorded_at,
        cast(json_value(event, '$.approved') as boolean) as approved,

    from {{ ref('stg_disbursal_events') }}
    where event_type = "approval_process_concluded"
)

, settled as (
    select
        id as disbursal_id,
        recorded_at as event_recorded_at,
        cast(json_value(event, '$.recorded_at') as timestamp) as settled_recorded_at,
        cast(json_value(event, '$.amount') as numeric) / {{ var('cents_per_usd') }} as settled_amount_usd,
        json_value(event, '$.ledger_tx_id') as ledger_tx_id,
        json_value(event, '$.obligation_id') as obligation_id,

    from {{ ref('stg_disbursal_events') }}
    where event_type = 'settled'
)

, final as (
    select *
    from initialized
    left join concluded using (disbursal_id)
    left join settled using (disbursal_id)
)


select * from final
