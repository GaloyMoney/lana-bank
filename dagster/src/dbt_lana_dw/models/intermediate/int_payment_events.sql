with

    payments as (select * from {{ ref("int_core_payment_events_rollup") }}),

    payment_allocations as (
        select
            payment_id,
            sum(amount_usd) as allocation_amount_usd,
            max(effective) as effective,
            max(payment_allocation_created_at) as payment_allocation_created_at,
            max(payment_allocation_modified_at) as payment_allocation_modified_at,
            array_agg(distinct obligation_type) as obligation_type
        from {{ ref("int_core_payment_allocation_events_rollup") }}
        group by payment_id
    ),

    final as (select * from payments left join payment_allocations using (payment_id))

select *
from final
