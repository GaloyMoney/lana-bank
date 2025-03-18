with approved as (

    select distinct id as credit_facility_id

    from {{ ref('stg_credit_facility_events') }}

    where
        event_type = 'approval_process_concluded'
    {% if target.type == 'bigquery' %}
        and json_value(event, '$.approved') = 'true'
    {% elif target.type == 'snowflake' %}
        and JSON_EXTRACT_PATH_TEXT(event, 'approved') = 'true'
    {% endif %}

),

initial as (

    select distinct
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        cast(json_value(event, '$.facility') as {{ dbt.type_numeric() }}) as facility,
        recorded_at as initialized_at,
        cast(json_value(event, '$.terms.annual_rate') as {{ dbt.type_numeric() }}) as annual_rate,
        case
            when json_value(event, '$.terms.duration.type') = 'months'
                then timestamp_add(
                    date(recorded_at),
                    interval cast(
                        json_value(event, '$.terms.duration.value') as {{ dbt.type_int() }}
                    ) month
                )
        end as end_date,
        json_value(event, '$.terms.incurrence_interval.type') as incurrence_interval,
        json_value(event, '$.terms.accrual_interval.type') as accrual_interval,
        json_value(event, '$.customer_id') as customer_id,
        json_value(event, '$.customer_account_ids.on_balance_sheet_deposit_account_id') as on_balance_sheet_deposit_account_id,
        json_value(event, '$.account_ids.collateral_account_id') as collateral_account_id,
        json_value(event, '$.account_ids.disbursed_receivable_account_id') as disbursed_receivable_account_id,
        json_value(event, '$.account_ids.facility_account_id') as facility_account_id,
        json_value(event, '$.account_ids.fee_income_account_id') as fee_income_account_id,
        json_value(event, '$.account_ids.interest_account_id') as interest_account_id,
        json_value(event, '$.account_ids.interest_receivable_account_id') as interest_receivable_account_id
    {% elif target.type == 'snowflake' %}
        cast(JSON_EXTRACT_PATH_TEXT(event, 'facility') as {{ dbt.type_numeric() }}) as facility,
        recorded_at as initialized_at,
        cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.annual_rate') as {{ dbt.type_numeric() }}) as annual_rate,
        case
            when JSON_EXTRACT_PATH_TEXT(event, 'terms.duration.type') = 'months'
                then TIMESTAMPADD(
                    month,
                    cast(JSON_EXTRACT_PATH_TEXT(event, 'terms.duration.value') as {{ dbt.type_int() }}),
                    date(recorded_at)
                )
        end as end_date,
        JSON_EXTRACT_PATH_TEXT(event, 'terms.incurrence_interval.type') as incurrence_interval,
        JSON_EXTRACT_PATH_TEXT(event, 'terms.accrual_interval.type') as accrual_interval,
        JSON_EXTRACT_PATH_TEXT(event, 'customer_id') as customer_id,
        JSON_EXTRACT_PATH_TEXT(event, 'customer_account_ids.on_balance_sheet_deposit_account_id') as on_balance_sheet_deposit_account_id,
        JSON_EXTRACT_PATH_TEXT(event, 'account_ids.collateral_account_id') as collateral_account_id,
        JSON_EXTRACT_PATH_TEXT(event, 'account_ids.disbursed_receivable_account_id') as disbursed_receivable_account_id,
        JSON_EXTRACT_PATH_TEXT(event, 'account_ids.facility_account_id') as facility_account_id,
        JSON_EXTRACT_PATH_TEXT(event, 'account_ids.fee_income_account_id') as fee_income_account_id,
        JSON_EXTRACT_PATH_TEXT(event, 'account_ids.interest_account_id') as interest_account_id,
        JSON_EXTRACT_PATH_TEXT(event, 'account_ids.interest_receivable_account_id') as interest_receivable_account_id
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'initialized'

),

payments as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        sum(cast(json_value(event, '$.interest_amount') as {{ dbt.type_numeric() }})) as total_interest_paid,
        sum(cast(json_value(event, '$.disbursal_amount') as {{ dbt.type_numeric() }})) as total_disbursement_paid,
        max(if(coalesce(cast(json_value(event, '$.interest_amount') as {{ dbt.type_numeric() }}), 0) > 0, recorded_at, null)) as most_recent_interest_payment_timestamp,
        max(if(coalesce(cast(json_value(event, '$.disbursal_amount') as {{ dbt.type_numeric() }}), 0) > 0, recorded_at, null)) as most_recent_disbursement_payment_timestamp
    {% elif target.type == 'snowflake' %}
        sum(cast(JSON_EXTRACT_PATH_TEXT(event, 'interest_amount') as {{ dbt.type_numeric() }})) as total_interest_paid,
        sum(cast(JSON_EXTRACT_PATH_TEXT(event, 'disbursal_amount') as {{ dbt.type_numeric() }})) as total_disbursement_paid,
        max(CASE WHEN coalesce(cast(JSON_EXTRACT_PATH_TEXT(event, 'interest_amount') as {{ dbt.type_numeric() }}), 0) > 0 THEN recorded_at ELSE null END) as most_recent_interest_payment_timestamp,
        max(CASE WHEN coalesce(cast(JSON_EXTRACT_PATH_TEXT(event, 'disbursal_amount') as {{ dbt.type_numeric() }}), 0) > 0 THEN recorded_at ELSE null END) as most_recent_disbursement_payment_timestamp
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'payment_recorded'

    group by credit_facility_id

),

interest as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        sum(cast(json_value(event, '$.amount') as {{ dbt.type_numeric() }})) as total_interest_incurred
    {% elif target.type == 'snowflake' %}
        sum(cast(JSON_EXTRACT_PATH_TEXT(event, 'amount') as {{ dbt.type_numeric() }})) as total_interest_incurred
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'interest_accrual_concluded'

    group by credit_facility_id

),

collateral as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        cast(json_value(any_value(event having max recorded_at), '$.total_collateral') as {{ dbt.type_numeric() }}) as total_collateral
    {% elif target.type == 'snowflake' %}
        cast(JSON_EXTRACT_PATH_TEXT(
            GET(MAX_BY(event, recorded_at, 1), 0),
            'total_collateral'
        ) as {{ dbt.type_numeric() }}) as total_collateral
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'collateral_updated'

    group by credit_facility_id

),

collateral_deposits as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        parse_timestamp('%Y-%m-%dT%H:%M:%E6SZ', json_value(any_value(event having max recorded_at), '$.recorded_at'), 'UTC') as most_recent_collateral_deposit
    {% elif target.type == 'snowflake' %}
        TO_TIMESTAMP(JSON_EXTRACT_PATH_TEXT(
            GET(MAX_BY(event, recorded_at, 1), 0),
            'recorded_at'
        )) as most_recent_collateral_deposit
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where
        event_type = 'collateral_updated'
    {% if target.type == 'bigquery' %}
        and json_value(event, '$.action') = 'Add'
    {% elif target.type == 'snowflake' %}
        and JSON_EXTRACT_PATH_TEXT(event, 'action') = 'Add'
    {% endif %}

    group by credit_facility_id

),

disbursements as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        sum(cast(json_value(event, '$.amount') as {{ dbt.type_numeric() }})) as total_disbursed
    {% elif target.type == 'snowflake' %}
        sum(cast(JSON_EXTRACT_PATH_TEXT(event, 'amount') as {{ dbt.type_numeric() }})) as total_disbursed
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'disbursal_initiated'

    group by credit_facility_id

),

completed as (

    select distinct id as credit_facility_id

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'completed'

)

select
    credit_facility_id,
    initialized_at,
    end_date,
    incurrence_interval,
    accrual_interval,
    most_recent_interest_payment_timestamp,
    most_recent_disbursement_payment_timestamp,
    annual_rate,
    customer_id,
    on_balance_sheet_deposit_account_id,
    collateral_account_id,
    disbursed_receivable_account_id,
    facility_account_id,
    fee_income_account_id,
    interest_account_id,
    interest_receivable_account_id,
    most_recent_collateral_deposit,
    row_number() over (order by null) as credit_facility_key,
    coalesce(facility, 0) as facility,
    coalesce(total_interest_paid, 0) as total_interest_paid,
    coalesce(total_disbursement_paid, 0) as total_disbursement_paid,
    coalesce(total_interest_incurred, 0) as total_interest_incurred,
    coalesce(total_collateral, 0) as total_collateral,
    coalesce(total_disbursed, 0) as total_disbursed,
    completed.credit_facility_id is not null as completed

from approved
inner join initial using (credit_facility_id)
left join payments using (credit_facility_id)
left join interest using (credit_facility_id)
left join collateral using (credit_facility_id)
left join collateral_deposits using (credit_facility_id)
left join disbursements using (credit_facility_id)
left join completed using (credit_facility_id)
