select
    'TODO' as {{ ident('id_codigo_cuentaproy') }},
    'TODO' as {{ ident('nom_cuentaproy') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 1 then cash_flow_amount end
        ),
        0
    ) as {{ ident('enero') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 2 then cash_flow_amount end
        ),
        0
    ) as {{ ident('febrero') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 3 then cash_flow_amount end
        ),
        0
    ) as {{ ident('marzo') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 4 then cash_flow_amount end
        ),
        0
    ) as {{ ident('abril') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 5 then cash_flow_amount end
        ),
        0
    ) as {{ ident('mayo') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 6 then cash_flow_amount end
        ),
        0
    ) as {{ ident('junio') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 7 then cash_flow_amount end
        ),
        0
    ) as {{ ident('julio') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 8 then cash_flow_amount end
        ),
        0
    ) as {{ ident('agosto') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 9 then cash_flow_amount end
        ),
        0
    ) as {{ ident('septiembre') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 10 then cash_flow_amount end
        ),
        0
    ) as {{ ident('octubre') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 11 then cash_flow_amount end
        ),
        0
    ) as {{ ident('noviembre') }},
    coalesce(
        sum(
            case when extract(month from period_end_date) = 12 then cash_flow_amount end
        ),
        0
    ) as {{ ident('diciembre') }}
from {{ ref("int_approved_credit_facility_loan_cash_flows") }}
where extract(year from period_end_date) = extract(year from current_timestamp())
