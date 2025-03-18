with loans_and_credit_facilities as (
    /* TODO:
    SELECT total_collateral,
    loan_id AS reference_id,
    principal AS loan_amount,

    FROM { ref('int_approved_loans') }

    WHERE NOT completed

    UNION ALL
    */
    select
        total_collateral,
        credit_facility_id as reference_id,
        facility as loan_amount

    from {{ ref('int_approved_credit_facilities') }}

    where not completed

)

select
    left(replace(upper(reference_id), '-', ''), 20) as "num_referencia",
    '{{ npb4_17_01_tipos_de_cartera(
        "Cartera propia Ley Acceso al Crédito (19)"
    ) }}' as "cod_cartera",
    '{{ npb4_17_02_tipos_de_activos_de_riesgo("Préstamos") }}' as "cod_activo",
    left(replace(upper(reference_id), '-', ''), 20)
        as "identificacion_garantia",
    '{{ npb4_17_09_tipos_de_garantias("Pignorada - Depósito de dinero") }}'
        as "tipo_garantia",
    coalesce((total_collateral * (
    {% if target.type == 'bigquery' %}
        select any_value(last_price_usd having max requested_at)
    {% elif target.type == 'snowflake' %}
        select GET(MAX_BY(last_price_usd, requested_at, 1), 0)
    {% endif %}
        from {{ ref('stg_bitfinex_ticker_price') }}
    ) / nullif(loan_amount * 100, 0)), 1) as "valor_garantia_proporcional"

from loans_and_credit_facilities
