select
    line_of_credit as `Línea de crédito`,
    disbursement_number as `Número de desembolso`,
    principal_balance as `Saldo de Capital o Principal`,
    interest as `Intereses`,
    total_debt as `Deuda Total`,
    guarantee_amount as `Monto de Garantía`,
    percent_by_risk_category as `% según categoria de riesgo`,
    category_b as `Categoria B`,
    net_risk as `Riesgo Neto`,
    reserve_percentage as `% de Reserva`,
    reserve as `Reserva`,
from {{ ref('int_net_risk_calculation') }}
