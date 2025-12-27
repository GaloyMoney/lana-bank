with

customers as (
    select *
    from {{ ref('int_sumsub_applicants') }}
),

statuses as (
    select
        ref.* except (customer_id, disbursal_id),
        estado,
        `explicación`,
        status,
        explanation,
        greatest(coalesce(dias_mora_k, 0), coalesce(dias_mora_i, 0)) as payment_overdue_days
    from {{ ref('int_nrp_41_02_referencia') }} as ref
    left join
        {{ ref('static_nrp_41_estados_del_prestamo') }}
        on
            greatest(
                coalesce(dias_mora_k, 0), coalesce(dias_mora_i, 0)
            ) between consumer_calendar_ge_days and consumer_calendar_le_days
),

loans as (
    select
        credit_facility_id as line_of_credit,
        disbursal_id as disbursement_number,
        disbursal_start_date as disbursement_date,
        annual_rate as interest_rate,
        customer_id,
        first_name || ' ' || last_name as customer_name,
        total_disbursed_usd as disbursed_amount,
        disbursal_end_date as maturity_date,
        coalesce(estado, 'Cancelado') as estado,
        `explicación`,
        coalesce(status, 'Canceled') as `status`,
        explanation,

        disbursal_start_date as date_and_time,
        'Disbursement' as `transaction`,
        total_disbursed_usd as principal,
        null as interest,
        null as fee,
        null as vat,
        total_disbursed_usd as total_transaction
    from {{ ref('int_approved_credit_facility_loans') }}
    left join customers using (customer_id)
    left join statuses using (credit_facility_id)
),

payments as (
    select
        loans.line_of_credit,
        loans.disbursement_number,
        loans.disbursement_date,
        loans.interest_rate,
        loans.customer_id,
        loans.customer_name,
        loans.disbursed_amount,
        loans.maturity_date,
        loans.estado,
        loans.`explicación`,
        loans.status,
        loans.explanation,

        payment_created_at as date_and_time,
        'Payment' as `transaction`,
        disbursal_usd as principal,
        interest_usd as interest,
        null as fee,
        null as vat,
        amount_usd as total_transaction
    from {{ ref('int_payment_history') }} as ph
    inner join loans on credit_facility_id = line_of_credit
),

final as (
    select * from loans
    union all
    select * from payments
)

select
    *,
    sum(total_transaction)
        over (partition by line_of_credit order by date_and_time)
        as principal_balance
from final
