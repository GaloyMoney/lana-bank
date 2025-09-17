select
    credit_facility.id as `Línea de crédito`,
    disbursement.id as `Número de desembolso`,
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
left join {{ ref('stg_core_public_ids') }} as credit_facility on line_of_credit = credit_facility.target_id
left join {{ ref('stg_core_public_ids') }} as disbursement on disbursement_number = disbursement.target_id
