with

credit_facilities as (
    select
        credit_facility_id,
        customer_id
    from {{ ref('int_core_credit_facility_events_rollup') }}
),

payments as (
    select *
    from {{ ref('int_core_payment_events_rollup') }}
),

final as (
    select *
    from credit_facilities
    -- sqlfluff: disable=convention.left_join
    right join payments using (credit_facility_id)
    -- sqlfluff: enable=convention.left_join

)

select *
from final
