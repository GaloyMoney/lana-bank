select
    disbursement_public_ids.id as `identificacion_garantia`,

    customer_public_ids.id as `nit_depositante`,

    disbursal_end_date as `fecha_vencimiento`,

    -- loan-to-collateral being 1-to-1
    collateral_amount_usd as `valor_deposito`,

    -- "DE" for cash deposits
    'DE' as `tipo_deposito`,

    null as `cod_banco`,

    -- Deposit date.
    date(most_recent_collateral_deposit_at) as `fecha_deposito`

from {{ ref('int_approved_credit_facility_loans') }}
left join
    {{ ref('stg_core_public_ids') }} as disbursement_public_ids
    on disbursal_id = disbursement_public_ids.target_id
left join
    {{ ref('stg_core_public_ids') }} as customer_public_ids
    on customer_id = customer_public_ids.target_id

where not matured
