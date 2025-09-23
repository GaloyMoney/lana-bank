with

loans as (
    select
        extract(month from period_end_date) as month,
        sum(cash_flow_amount) as monthly_cash_flow_amount
    from {{ ref('int_approved_credit_facility_loan_cash_flows') }}
    where extract(year from period_end_date) = extract(year from current_timestamp())
    group by extract(month from period_end_date)
)
,

final as (

    select
        (
            select monthly_cash_flow_amount from loans
            where month = 1
        ) as jan,
        (
            select monthly_cash_flow_amount from loans
            where month = 2
        ) as feb,
        (
            select monthly_cash_flow_amount from loans
            where month = 3
        ) as mar,
        (
            select monthly_cash_flow_amount from loans
            where month = 4
        ) as apr,
        (
            select monthly_cash_flow_amount from loans
            where month = 5
        ) as may,
        (
            select monthly_cash_flow_amount from loans
            where month = 6
        ) as jun,
        (
            select monthly_cash_flow_amount from loans
            where month = 7
        ) as jul,
        (
            select monthly_cash_flow_amount from loans
            where month = 8
        ) as aug,
        (
            select monthly_cash_flow_amount from loans
            where month = 9
        ) as sep,
        (
            select monthly_cash_flow_amount from loans
            where month = 10
        ) as oct,
        (
            select monthly_cash_flow_amount from loans
            where month = 11
        ) as nov,
        (
            select monthly_cash_flow_amount from loans
            where month = 12
        ) as dec
)

select
    'TODO' as `id_codigo_cuentaproy`,
    'TODO' as `nom_cuentaproy`,
    coalesce(jan, 0) as `enero`,
    coalesce(feb, 0) as `febrero`,
    coalesce(mar, 0) as `marzo`,
    coalesce(apr, 0) as `abril`,
    coalesce(may, 0) as `mayo`,
    coalesce(jun, 0) as `junio`,
    coalesce(jul, 0) as `julio`,
    coalesce(aug, 0) as `agosto`,
    coalesce(sep, 0) as `septiembre`,
    coalesce(oct, 0) as `octubre`,
    coalesce(nov, 0) as `noviembre`,
    coalesce(dec, 0) as `diciembre`
from
    final
