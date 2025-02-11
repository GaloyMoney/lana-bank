with disbursal_inits as (

    select
        id as credit_facility_id,
        json_value(parsed_event.idx) as disbursal_idx,
        lax_int64(parsed_event.amount) / 100 as disbursal_amount_usd

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_initiated"

),

disbursal_concludes as (

    select
        id as credit_facility_id,
        date(recorded_at) as day,
        json_value(parsed_event.idx) as disbursal_idx,
        lax_bool(parsed_event.canceled) as disbursal_canceled

    from {{ ref('stg_credit_facility_events') }}

    where event_type = "disbursal_concluded"

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
    countif(not disbursal_canceled) as approved_n_disbursals

from disbursal_inits
inner join disbursal_concludes using (credit_facility_id, disbursal_idx)

group by credit_facility_id, day
