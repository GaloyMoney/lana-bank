with approved as (

    select distinct id as credit_facility_id

    from {{ ref('stg_credit_facility_events') }}

    where
        event_type = "approval_process_concluded"
        and json_value(event, "$.approved") = "true"

),

initial as (

    select distinct
        id as credit_facility_id,
        cast(json_value(event, "$.facility") as numeric) as facility,
        recorded_at as credit_facility_initialized_at,
        case
            when json_value(event, "$.terms.duration.type") = "months"
                then timestamp_add(
                    date(recorded_at),
                    interval cast(
                        json_value(event, "$.terms.duration.value") as integer
                    ) month
                )
        end as end_date,
        cast(json_value(event, "$.terms.annual_rate") as numeric) as annual_rate,
        json_value(event, "$.terms.duration.type") as duration_type,
        json_value(event, "$.terms.duration.value") as duration_value,
        json_value(event, "$.terms.accrual_interval.type") as accrual_interval,
        json_value(event, "$.terms.accrual_cycle_interval.type") as accrual_cycle_interval,
        json_value(event, "$.customer_id") as customer_id,
        json_value(
            event, "$.customer_account_ids.on_balance_sheet_deposit_account_id"
        ) as on_balance_sheet_deposit_account_id,
        json_value(event, "$.account_ids.collateral_account_id")
            as collateral_account_id,
        json_value(event, "$.account_ids.disbursed_receivable_account_id")
            as disbursed_receivable_account_id,
        json_value(event, "$.account_ids.facility_account_id")
            as facility_account_id,
        json_value(event, "$.account_ids.fee_income_account_id")
            as fee_income_account_id,
        json_value(event, "$.account_ids.interest_account_id")
            as interest_account_id,
        json_value(event, "$.account_ids.interest_receivable_account_id")
            as interest_receivable_account_id

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "initialized"

),

disbursal_concluded as (

    select
        id as credit_facility_id,
        recorded_at as disbursal_concluded_at,
        cast(json_value(event, "$.idx") as integer) as idx,
        json_value(event, "$.tx_id") as tx_id,
        json_value(event, "$.obligation_id") as obligation_id,

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_concluded"

),

disbursal_initiated as (

    select
        id as credit_facility_id,
        recorded_at as disbursal_initialized_at,
        cast(json_value(event, "$.idx") as integer) as idx,
        cast(json_value(event, "$.amount") as numeric) as total_disbursed,
        json_value(event, "$.disbursal_id") as disbursal_id,
        json_value(event, "$.approval_process_id") as approval_process_id,

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_initiated"

),

completed as (

    select distinct id as credit_facility_id

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "completed"

),

final as(

    select
        credit_facility_id,
        customer_id,
        credit_facility_initialized_at,
        disbursal_initialized_at,
        disbursal_concluded_at,
        disbursal_concluded_at as start_date,
        end_date,
        duration_value,
        duration_type,
        annual_rate,
        accrual_interval,
        accrual_cycle_interval,
        on_balance_sheet_deposit_account_id,
        collateral_account_id,
        disbursed_receivable_account_id,
        facility_account_id,
        fee_income_account_id,
        interest_account_id,
        interest_receivable_account_id,
        tx_id,
        obligation_id,
        disbursal_id,
        approval_process_id,
        coalesce(facility, 0) as facility,
        coalesce(total_disbursed, 0) as total_disbursed,
        completed.credit_facility_id is not null as completed

    from approved
    inner join initial using (credit_facility_id)
        join disbursal_concluded using (credit_facility_id)
    left join disbursal_initiated using (credit_facility_id, idx)
    left join completed using (credit_facility_id)
)


select * from final
