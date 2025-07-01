with

credit_facilities as(
    select
        credit_facility_id,
        customer_id,
    from {{ ref('int_core_credit_facility_events_rollup') }}
)

, payments as(
    select * except(is_payment_allocated, audit_entry_ids)
    from {{ ref('int_core_payment_events_rollup') }}
)

, payment_allocations as(
    select
        payment_id,
        max(effective) as effective,
        array_agg(distinct obligation_type) as obligation_type,
    from {{ ref('int_core_payment_allocation_events_rollup') }}
    group by payment_id
)

, final as (
    select
        *
    from credit_facilities
    right join payments using(credit_facility_id)

)

select *
from final
