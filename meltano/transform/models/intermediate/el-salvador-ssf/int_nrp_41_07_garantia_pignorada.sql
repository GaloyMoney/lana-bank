select
    -- uses the 20 leftmost no-hyphen characters from backend loan_id
    -- loan-to-collateral being 1-to-1
    left(replace(upper(disbursal_id), '-', ''), 20) as `identificacion_garantia`,

    left(replace(customer_id, '-', ''), 14) as `nit_depositante`,

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

where not matured
