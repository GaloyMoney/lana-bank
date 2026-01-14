select
    credit_facility_public_ids.id as `Línea de crédito`,
    disbursement_public_ids.id as `Número de desembolso`,
    disbursement_date as `Fecha de desembolso`,
    interest_rate as `Tasa de interés`,
    customer_name as `Nombre del cliente`,
    disbursed_amount as `Monto desembolsado`,
    maturity_date as `Fecha de vencimiento`,
    date_and_time as `Fecha y hora`,
    transaction as `Transacción`,
    principal as `Principal`,
    interest as `Interes`,
    fee as `Comisión`,
    vat as `IVA`,
    total_transaction as `Total transacción`,
    principal_balance as `Saldo Principal`,
    coalesce(estado, 'Cancelado') as `Estado`
from {{ ref('int_loan_statements') }}
left join
    {{ ref('stg_core_public_ids') }} as credit_facility_public_ids
    on line_of_credit = credit_facility_public_ids.target_id
left join
    {{ ref('stg_core_public_ids') }} as disbursement_public_ids
    on disbursement_number = disbursement_public_ids.target_id
order by date_and_time
