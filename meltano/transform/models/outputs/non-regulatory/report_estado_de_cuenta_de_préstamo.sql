select
    line_of_credit as `Línea de crédito`,
    disbursement_number as `Número de desembolso`,
    disbursement_date as `Fecha de desembolso`,
    interest_rate as `Tasa de interés`,
    customer_name as `Nombre del cliente`,
    disbursed_amount as `Monto desembolsado`,
    maturity_date as `Fecha de vencimiento`,
    coalesce(estado, 'Cancelado') as `Estado`,
    date_and_time as `Fecha y hora`,
    transaction as `Transacción`,
    principal as `Principal`,
    interest as `Interes`,
    fee as `Comisión`,
    vat as `IVA`,
    total_transaction as `Total transacción`,
    principal_balance as `Saldo Principal`,
from {{ ref('int_loan_statements') }}
order by date_and_time
