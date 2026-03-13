select
    'TODO' as `id_codigo_cuentaproy`,
    'TODO' as `nom_cuentaproy`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 1 then cash_flow_amount end
        ),
        0
    ) as `enero`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 2 then cash_flow_amount end
        ),
        0
    ) as `febrero`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 3 then cash_flow_amount end
        ),
        0
    ) as `marzo`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 4 then cash_flow_amount end
        ),
        0
    ) as `abril`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 5 then cash_flow_amount end
        ),
        0
    ) as `mayo`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 6 then cash_flow_amount end
        ),
        0
    ) as `junio`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 7 then cash_flow_amount end
        ),
        0
    ) as `julio`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 8 then cash_flow_amount end
        ),
        0
    ) as `agosto`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 9 then cash_flow_amount end
        ),
        0
    ) as `septiembre`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 10 then cash_flow_amount end
        ),
        0
    ) as `octubre`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 11 then cash_flow_amount end
        ),
        0
    ) as `noviembre`,
    coalesce(
        sum(
            case when extract(month from period_end_date) = 12 then cash_flow_amount end
        ),
        0
    ) as `diciembre`
from {{ ref("int_approved_credit_facility_loan_cash_flows") }}
where extract(year from period_end_date) = extract(year from current_timestamp())
