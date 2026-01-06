with

    customers as (select * from {{ ref("int_sumsub_applicants") }}),

    statuses as (
        select
            ref.* except (customer_id, disbursal_id),
            estado,
            `explicación`,
            status,
            explanation,
            coalesce(dias_mora_k, 0) as days_past_due_on_principal,
            coalesce(dias_mora_i, 0) as days_past_due_on_interest,
            greatest(
                coalesce(dias_mora_k, 0), coalesce(dias_mora_i, 0)
            ) as payment_overdue_days
        from {{ ref("int_nrp_41_02_referencia") }} as ref
        left join
            {{ ref("static_nrp_41_estados_del_prestamo") }}
            on greatest(coalesce(dias_mora_k, 0), coalesce(dias_mora_i, 0))
            between consumer_calendar_ge_days and consumer_calendar_le_days
    ),

    loans as (
        select
            credit_facility_id as line_of_credit,
            disbursal_id as disbursement_number,
            disbursal_start_date as disbursement_date,
            annual_rate as interest_rate,
            customer_id,
            total_disbursed_usd as disbursed_amount,
            disbursal_end_date as maturity_date,
            days_past_due_on_principal,

            days_past_due_on_interest,
            payment_overdue_days,
            `explicación`,
            explanation,
            disbursal_start_date as date_and_time,
            'Disbursement' as transaction,
            total_disbursed_usd as principal,

            null as interest,
            null as fee,
            null as vat,
            total_disbursed_usd as total_transaction,
            first_name || ' ' || last_name as customer_name,
            coalesce(estado, 'Cancelado') as estado,
            coalesce(status, 'Canceled') as status
        from {{ ref("int_approved_credit_facility_loans") }}
        left join customers using (customer_id)
        left join statuses using (credit_facility_id)
    ),

    risk as (select * from {{ ref("int_net_risk_calculation") }}),

    final as (
        select * from loans left join risk using (line_of_credit, disbursement_number)
    )

select
    line_of_credit as credit_line_no,
    disbursement_number,
    1 as product_code,
    'PRESTAMOS GARANTIZADOS CON BITCOIN' as product,
    customer_id as customer_code,
    customer_name,
    disbursement_date,
    maturity_date,
    status,
    interest_rate,
    interest_rate as effective_rate,
    disbursed_amount,
    principal_balance,
    0 as interest_balance,
    days_past_due_on_principal,
    days_past_due_on_interest,
    'Privado' as credit_type,
    category_b as risk_rating,
    'Automática' as category_assignment_type,
    net_risk,
    reserve as reserve_amount,
    '1141030101' as previous_equity_account,
    '1141030101' as new_equity_account,
    '915000' as new_interest_account,
    payment_overdue_days / 30 as number_of_past_due_installments,
    100 * guarantee_amount / coalesce(principal_balance, 1) as guaranteed_percentage
from final
