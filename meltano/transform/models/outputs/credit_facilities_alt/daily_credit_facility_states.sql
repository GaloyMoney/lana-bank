{{ config(materialized='table') }}

{% if target.type == 'bigquery' %}
{% set target_database = '' %}
{% elif target.type == 'snowflake' %}
{% set target_database = target.database + '.' %}
{% endif %}

-- TODO: these daily states should be derived from an instantaneous state table
-- this would reduce any potential inconsistencies due to the aggregation.
-- It can be done in a future PR if it turns out the interface works.
with approvals as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
    {% if target.type == 'bigquery' %}
        lax_bool(parsed_event.approved) as approved
    {% elif target.type == 'snowflake' %}
        cast(PARSE_JSON(event):approved as {{ dbt.type_boolean() }}) as approved
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'approval_process_concluded'


),

payments as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
    {% if target.type == 'bigquery' %}
        sum(lax_int64(parsed_event.disbursal_amount)) / 100 as disbursal_amount_paid_usd,
        sum(lax_int64(parsed_event.interest_amount)) / 100 as interest_amount_paid_usd,
    {% elif target.type == 'snowflake' %}
        sum(cast(PARSE_JSON(event):disbursal_amount as {{ dbt.type_int() }})) / 100 as disbursal_amount_paid_usd,
        sum(cast(PARSE_JSON(event):interest_amount as {{ dbt.type_int() }})) / 100 as interest_amount_paid_usd,
    {% endif %}
        count(*) as n_payments

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'payment_recorded'

    group by credit_facility_id, day

),

interest as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
    {% if target.type == 'bigquery' %}
        lax_int64(parsed_event.amount) / 100 as interest_incurred_usd
    {% elif target.type == 'snowflake' %}
        cast(PARSE_JSON(event):amount as {{ dbt.type_int() }}) / 100 as interest_incurred_usd
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'interest_accrual_concluded'

),

completions as (

    select distinct
        id as credit_facility_id,
        true as completed,
        date(recorded_at) as completed_on

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'completed'

),

{% if target.type == 'bigquery' %}
active_days as (

    select
        credit_facility_id,
        true as active,
        date(day) as day

    from (

        select
            credit_facility_id,
            generate_timestamp_array(
                timestamp(day),
                coalesce(timestamp(completed_on), current_timestamp()),
                interval 1 day
            ) as days

        from approvals
        left join completions using (credit_facility_id)

    ), unnest(days) as day
),
{% elif target.type == 'snowflake' %}
active_days_pre as (
    select
        credit_facility_id,
        true as active,
        a.day as start_date,
        (
            SELECT
                ARRAY_AGG("DAY") A
            FROM (
                SELECT ROW_NUMBER() OVER (ORDER BY NULL) - 1 "DAY"
                FROM TABLE(GENERATOR(ROWCOUNT => 1000))
            )
            WHERE "DAY" <= CEIL(DATEDIFF(DAYS, a.day, last_day(
                coalesce(c.completed_on, current_date()))))
        ) as date_offsets

    from approvals a
    left join completions c using (credit_facility_id)
),
active_days as (
    select
        credit_facility_id,
        active,
        TO_TIMESTAMP(DATEADD(DAY, days.value, start_date)) as day

    from active_days_pre, table(flatten(input => date_offsets, mode => 'array')) as days
),
{% endif %}

joined as (

    select
        day,
        credit_facility_id,
        initial_price_usd_per_btc,
        coalesce(active, false) as active,
        coalesce(
            last_value(close_price_usd_per_btc ignore nulls) over (
                order by day asc
            ),
            last_value(close_price_usd_per_btc ignore nulls) over (
                order by day desc
            )
        ) as close_price_usd_per_btc,
        coalesce(disbursal_amount_usd, 0) as disbursal_amount_usd,
        coalesce(n_disbursals, 0) as n_disbursals,
        coalesce(approved_disbursal_amount_usd, 0) as approved_disbursal_amount_usd,
        coalesce(approved_n_disbursals, 0) as approved_n_disbursals,
        coalesce(disbursal_amount_paid_usd, 0) as disbursal_amount_paid_usd,
        coalesce(interest_amount_paid_usd, 0) as interest_amount_paid_usd,
        coalesce(n_payments, 0) as n_payments,
        coalesce(interest_incurred_usd, 0) as interest_incurred_usd,
        coalesce(
            last_value(total_collateral_btc ignore nulls) over (
                partition by credit_facility_id
                order by day
            ), 0
        ) as total_collateral_btc

    from {{ ref('int_days') }}
    full join active_days using (day)
    full join {{ ref('int_credit_facility_disbursals') }} using (credit_facility_id, day)
    full join payments using (credit_facility_id, day)
    full join interest using (credit_facility_id, day)
    full join {{ ref('int_credit_facility_collateral') }} using (credit_facility_id, day)

),

filled as (

    select
    {% if target.type == 'bigquery' %}
        joined.* except (initial_price_usd_per_btc),
        coalesce(initial_price_usd_per_btc, close_price_usd_per_btc) as initial_price_usd_per_btc,
        sum(approved_disbursal_amount_usd) over (past) as total_disbursed_usd,
        sum(approved_n_disbursals) over (past) as total_n_disbursals,
        sum(disbursal_amount_paid_usd) over (past) as total_disbursal_amount_paid_usd,
        sum(interest_amount_paid_usd) over (past) as total_interest_amount_paid_usd,
        sum(n_payments) over (past) as total_n_payments,
        sum(interest_incurred_usd) over (past) as total_interest_incurred_usd,
        total_collateral_btc - lag(total_collateral_btc, 1, 0) over (past) as collateral_change_btc

    from joined

    window
        past as (
            partition by credit_facility_id
            order by day
        )
    {% elif target.type == 'snowflake' %}
        joined.* exclude (initial_price_usd_per_btc),
        coalesce(initial_price_usd_per_btc, close_price_usd_per_btc) as initial_price_usd_per_btc,
        sum(approved_disbursal_amount_usd) over (partition by credit_facility_id order by day) as total_disbursed_usd,
        sum(approved_n_disbursals) over (partition by credit_facility_id order by day) as total_n_disbursals,
        sum(disbursal_amount_paid_usd) over (partition by credit_facility_id order by day) as total_disbursal_amount_paid_usd,
        sum(interest_amount_paid_usd) over (partition by credit_facility_id order by day) as total_interest_amount_paid_usd,
        sum(n_payments) over (partition by credit_facility_id order by day) as total_n_payments,
        sum(interest_incurred_usd) over (partition by credit_facility_id order by day) as total_interest_incurred_usd,
        total_collateral_btc - lag(total_collateral_btc, 1, 0) over (partition by credit_facility_id order by day) as collateral_change_btc

    from joined
    {% endif %}

),

{% if target.type == 'bigquery' %}
avg_open_price as (

    select
        credit_facility_id,
        day,
        avg_open_prices[o] as collateral_avg_open_price

    from (

        select
            credit_facility_id,
            array_agg(
                day
                order by day
            ) as days,
            {{ target_database }}{{ target.schema }}.udf_avg_open_price(
                array_agg(
                    collateral_change_btc
                    order by day
                ),
                array_agg(
                    initial_price_usd_per_btc
                    order by day
                )
            ) as avg_open_prices

        from filled

        group by credit_facility_id

    ), unnest(days) as day with offset as o

)
{% elif target.type == 'snowflake' %}
avg_open_price_pre as (

    select
        credit_facility_id,
        array_agg(day) WITHIN GROUP (order by day) as days,
        {{ target_database }}{{ target.schema }}.udf_avg_open_price(
            array_agg(collateral_change_btc) WITHIN GROUP (order by day),
            array_agg(initial_price_usd_per_btc) WITHIN GROUP (order by day)
        ) as avg_open_prices

    from filled
    group by credit_facility_id

),
avg_open_price as (

    select
        credit_facility_id,
        days.value as day,
        avg_open_prices[days.index] as collateral_avg_open_price

    from avg_open_price_pre, table(flatten(input => days, mode => 'array')) as days

)
{% endif %}

select
    *,
    sum(collateral_change_btc * initial_price_usd_per_btc)
        over (
            partition by credit_facility_id
            order by day
        )
        as initial_collateral_value_usd,
    total_collateral_btc * close_price_usd_per_btc as total_collateral_value_usd,
    day = max(day) over () as today

from filled
left join avg_open_price using (credit_facility_id, day)
