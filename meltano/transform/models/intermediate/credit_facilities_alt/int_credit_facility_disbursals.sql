with disbursal_inits as (

    select
        id as credit_facility_id,
    {% if target.type == 'bigquery' %}
        json_value(parsed_event.idx) as disbursal_idx,
        lax_int64(parsed_event.amount) / 100 as disbursal_amount_usd
    {% elif target.type == 'snowflake' %}
        JSON_EXTRACT_PATH_TEXT(event, 'idx') as disbursal_idx,
        cast(JSON_EXTRACT_PATH_TEXT(event, 'amount') as {{ dbt.type_int() }}) / 100 as disbursal_amount_usd
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'disbursal_initiated'

),

disbursal_concludes as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
    {% if target.type == 'bigquery' %}
        json_value(parsed_event.idx) as disbursal_idx,
        lax_bool(parsed_event.canceled) as disbursal_canceled
    {% elif target.type == 'snowflake' %}
        JSON_EXTRACT_PATH_TEXT(event, 'idx') as disbursal_idx,
        cast(JSON_EXTRACT_PATH_TEXT(event, 'canceled') as {{ dbt.type_boolean() }}) as disbursal_canceled
    {% endif %}

    from {{ ref('stg_credit_facility_events') }}

    where event_type = 'disbursal_concluded'

)


select
    credit_facility_id,
    day,
    sum(disbursal_amount_usd) as disbursal_amount_usd,
    count(distinct disbursal_idx) as n_disbursals,
    sum(
        case
            when disbursal_canceled then 0
            else disbursal_amount_usd
        end
    ) as approved_disbursal_amount_usd,
{% if target.type == 'bigquery' %}
    countif(not disbursal_canceled) as approved_n_disbursals
{% elif target.type == 'snowflake' %}
    count_if(not disbursal_canceled) as approved_n_disbursals
{% endif %}

from disbursal_inits
inner join disbursal_concludes using (credit_facility_id, disbursal_idx)

group by credit_facility_id, day
