with disbursal_inits as (

    select
        id as credit_facility_id,
        json_value(parsed_event.disbursal_id) as disbursal_id,
        lax_int64(parsed_event.amount) / 100 as disbursal_amount_usd

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_initiated"

),

disbursal_concludes as (

    select
        id as credit_facility_id,
        json_value(parsed_event.disbursal_id) as disbursal_id,
        recorded_at,
        lax_bool(parsed_event.canceled) as disbursal_canceled

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_concluded"

)

select
    credit_facility_id,
    disbursal_id,
    recorded_at,
    disbursal_amount_usd,
    case
        when disbursal_canceled then 0
        else disbursal_amount_usd
    end as approved_disbursal_amount_usd,
    not disbursal_canceled as approved

from disbursal_inits
inner join disbursal_concludes using (credit_facility_id, disbursal_id)
