with approved_credit_facilities as (
    select
        *
    from {{ ref('int_credit_facility_events_combo') }}
    where approved
),

disbursals as (
    select
        credit_facility_id,
        initialized_recorded_at as disbursal_initialized_at,
        initialized_recorded_at as disbursal_concluded_at,
        initialized_amount as total_disbursed,
        disbursal_id,
        obligation_id,
    from {{ ref('int_disbursal_events') }}
),

final as(

    select
        credit_facility_id,
        customer_id,
        initialized_recorded_at as credit_facility_initialized_at,
        disbursal_initialized_at,
        disbursal_concluded_at,
        disbursal_concluded_at as start_date,
        maturity_at as end_date,
        duration_value,
        duration_type,
        annual_rate,
        accrual_interval,
        accrual_cycle_interval,
        disbursal_id,
        obligation_id,
        coalesce(facility_amount, 0) as facility,
        coalesce(total_disbursed, 0) as total_disbursed,
        maturity_at < current_date() as matured

    from approved_credit_facilities
        join disbursals using (credit_facility_id)
)


select * from final
