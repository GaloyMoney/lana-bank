select
    credit_facility_public_ids.id as `No Linea de crédito`,
    disbursement_public_ids.id as `Número de desembolso`,
    product_code as `Cod Producto`,
    product as `Producto`,
    customer_public_ids.id as `Código de Cliente`,
    customer_name as `Nombre del cliente`,
    disbursement_date as `Fecha de desembolso`,
    maturity_date as `Fecha de vencimiento`,
    status as `Estado`,
    interest_rate as `Tasa de interés`,
    effective_rate as `Tasa efectiva`,
    disbursed_amount as `Monto desembolsado`,
    principal_balance as `Saldo de Principal`,
    interest_balance as `Saldo de Interés`,
    days_past_due_on_principal as `Días mora de capital`,
    days_past_due_on_interest as `Días mora de intereses`,
    number_of_past_due_allocations as `Número de asignaciones con atraso`,
    type_of_credit as `Tipo de crédito`,
    risk_rating as `Calificación de riesgo`,
    category_assignment_type as `Tipo de asignación de categoria`,
    percentage_guaranteed as `% Garantizado`,
    net_risk as `Riesgo neto`,
    reserve_amount as `Monto de Reserva`,
    capital_account as `Cuenta contable Capital`,
    interest_account as `Cuenta contable Interes`
from {{ ref("int_loan_portfolio") }}
left join
    {{ ref("stg_core_public_ids") }} as credit_facility_public_ids
    on line_of_credit_no = credit_facility_public_ids.target_id
left join
    {{ ref("stg_core_public_ids") }} as disbursement_public_ids
    on disbursement_number = disbursement_public_ids.target_id
left join
    {{ ref("stg_core_public_ids") }} as customer_public_ids
    on customer_code = customer_public_ids.target_id
