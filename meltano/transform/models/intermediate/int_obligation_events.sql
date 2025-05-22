with initialized as (
    select
        id as obligation_id,
        recorded_at as event_recorded_at,
        cast(json_value(event, "$.recorded_at") as timestamp) as initialized_recorded_at,

        json_value(event, "$.credit_facility_id") as credit_facility_id,

        json_value(event, "$.obligation_type") as obligation_type,
        cast(json_value(event, '$.amount') as numeric) / {{ var('cents_per_usd') }} as facility_amount_usd,

        cast(json_value(event, "$.due_date") as timestamp) as due_date,
        cast(json_value(event, "$.overdue_date") as timestamp) as overdue_date,
        cast(json_value(event, "$.defaulted_date") as timestamp) as defaulted_date,

        json_value(event, "$.tx_id") as initialized_tx_id,

        json_value(event, "$.defaulted_account_id") as defaulted_account_id,
        json_value(event, "$.due_accounts.receivable_account_id") as due_accounts_receivable_account_id,
        json_value(event, "$.due_accounts.account_to_be_credited_id") as due_accounts_account_to_be_credited_id,
        json_value(event, "$.overdue_accounts.receivable_account_id") as overdue_accounts_receivable_account_id,
        json_value(event, "$.overdue_accounts.account_to_be_credited_id") as overdue_accounts_account_to_be_credited_id,
        json_value(event, "$.not_yet_due_accounts.receivable_account_id") as not_yet_due_accounts_receivable_account_id,
        json_value(event, "$.not_yet_due_accounts.account_to_be_credited_id") as not_yet_due_accounts_account_to_be_credited_id,

    from {{ ref('stg_obligation_events') }}
    where event_type = 'initialized'
)

, due_recorded as (
    select
        id as obligation_id,
        recorded_at as due_recorded_at,

        cast(json_value(event, '$.amount') as numeric) as due_recorded_amount,

        json_value(event, "$.tx_id") as due_recorded_tx_id,

    from {{ ref('stg_obligation_events') }}
    where event_type = "due_recorded"
)

, overdue_recorded as (
    select
        id as obligation_id,
        recorded_at as overdue_recorded_at,

        cast(json_value(event, '$.amount') as numeric) as overdue_recorded_amount,

        json_value(event, "$.tx_id") as overdue_recorded_tx_id,

    from {{ ref('stg_obligation_events') }}
    where event_type = "overdue_recorded"
)

, payment_allocated as (
    select
        id as obligation_id,
        recorded_at as payment_allocated_at,

        cast(json_value(event, '$.amount') as numeric) as payment_allocated_amount,

        json_value(event, "$.tx_id") as payment_allocated_tx_id,

        json_value(event, "$.payment_id") as payment_id,
        json_value(event, "$.payment_allocation_id") as payment_allocation_id,

    from {{ ref('stg_obligation_events') }}
    where event_type = "payment_allocated"
)

, final as (
    select
        *
    from initialized
    left join due_recorded using (obligation_id)
    left join overdue_recorded using (obligation_id)
    left join payment_allocated using (obligation_id)
)


select * from final
