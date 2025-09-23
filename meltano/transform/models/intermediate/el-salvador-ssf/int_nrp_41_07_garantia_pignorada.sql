select
    -- loan-to-collateral being 1-to-1
    disbursement_public_ids.id as `identificacion_garantia`,

    customer_public_ids.id as `nit_depositante`,

    -- Deposit date.
    disbursal_end_date as `fecha_vencimiento`,

    -- Due date of the deposit.
    collateral_amount_usd as `valor_deposito`,
    'DE' as `tipo_deposito`,

    -- "DE" for cash deposits
    'BC99' as `cod_banco`,

    -- "BC99" for a yet undefined lana bank
    date(most_recent_collateral_deposit_at) as `fecha_deposito`

from {{ ref('int_approved_credit_facility_loans') }}
left join
    {{ ref('stg_core_public_ids') }} as disbursement_public_ids
    on disbursal_id = disbursement_public_ids.target_id
left join
    {{ ref('stg_core_public_ids') }} as customer_public_ids
    on customer_id = customer_public_ids.target_id

where not matured
