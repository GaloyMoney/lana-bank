select
    -- uses the 20 leftmost no-hyphen characters from backend loan_id
    -- loan-to-collateral being 1-to-1
    disbursal_end_date as `fecha_vencimiento`,

    collateral_amount_usd as `valor_deposito`,

    -- Deposit date.
    'DE' as `tipo_deposito`,

    -- Due date of the deposit.
    'BC99' as `cod_banco`,
    left(replace(upper(disbursal_id), '-', ''), 20) as `identificacion_garantia`,

    -- "DE" for cash deposits
    left(replace(customer_id, '-', ''), 14) as `nit_depositante`,

    -- "BC99" for a yet undefined lana bank
    date(most_recent_collateral_deposit_at) as `fecha_deposito`

from {{ ref('int_approved_credit_facility_loans') }}

where not matured
