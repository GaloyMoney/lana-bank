with

payments as (
    select *
    from {{ ref('int_core_payment_events_rollup') }}
),

obligation_installments as (
    select
        payment_id,
        sum(amount_usd) as allocation_amount_usd,
        max(effective) as effective,
        max(obligation_installment_created_at) as obligation_installment_created_at,
        max(obligation_installment_modified_at) as obligation_installment_modified_at,
        array_agg(distinct obligation_type) as obligation_type
    from {{ ref('int_core_obligation_installment_events_rollup') }}
    group by payment_id
),

final as (
    select *
    from payments
    left join obligation_installments using (payment_id)
)

select * from final
