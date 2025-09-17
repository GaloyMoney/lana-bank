select
    -- loan-to-collateral being 1-to-1
    disbursement.id as `identificacion_garantia`,

    customer.id as `nit_depositante`,

    -- Deposit date.
    date(most_recent_collateral_deposit_at) as `fecha_deposito`,

    -- Due date of the deposit.
    disbursal_end_date as `fecha_vencimiento`,
    collateral_amount_usd  as `valor_deposito`,

    -- "DE" for cash deposits
    'DE' as `tipo_deposito`,

    -- "BC99" for a yet undefined lana bank
    'BC99' as `cod_banco`

from {{ ref('int_approved_credit_facility_loans') }}
left join {{ ref('stg_core_public_ids') }} as disbursement on disbursal_id = disbursement.target_id
left join {{ ref('stg_core_public_ids') }} as customer on customer_id = customer.target_id

where not matured
