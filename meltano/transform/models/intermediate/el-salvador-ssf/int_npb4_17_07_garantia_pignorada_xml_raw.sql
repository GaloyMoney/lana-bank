select
    -- uses the 20 leftmost no-hyphen characters from backend loan_id
    -- loan-to-collateral being 1-to-1
    left(replace(upper(credit_facility_id), '-', ''), 20) as "identificacion_garantia",

    left(replace(customer_id, '-', ''), 14) as "nit_depositante",

    -- Deposit date.
    date(most_recent_collateral_deposit) as "fecha_deposito",

    -- Due date of the deposit.
    end_date as "fecha_vencimiento",
    (total_collateral * (
    {% if target.type == 'bigquery' %}
        select any_value(last_price_usd having max requested_at)
    {% elif target.type == 'snowflake' %}
        select GET(MAX_BY(last_price_usd, requested_at, 1), 0)
    {% endif %}
        from {{ ref('stg_bitfinex_ticker_price') }}
    )) as "valor_deposito",

    -- "DE" for cash deposits
    'DE' as "tipo_deposito",

    -- "BC99" for a yet undefined lana bank
    'BC99' as "cod_banco"

from {{ ref('int_approved_credit_facilities') }}

where not completed
