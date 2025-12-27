with

deposit_balances as (
    select *
    from
        {{ ref('int_deposit_balances') }}
)
,

deposit_accounts as (
    select *
    from
        {{ ref('int_core_deposit_account_events_rollup') }}
)
,

customers as (
    select *
    from
        {{ ref('int_core_customer_events_rollup') }}
)
,

approved_credit_facilities as (
    select *
    from
        {{ ref('int_approved_credit_facilities') }}
)
,

btc_price as (

    select any_value(last_price_usd having max requested_at) as last_price_usd
    from {{ ref('stg_bitfinex_ticker_price') }}

)
,

final as (

    select *
    from deposit_balances
    left join deposit_accounts using (deposit_account_id)
    left join customers using (customer_id)
    left join approved_credit_facilities using (customer_id)
)

select
    'BTCL' as `Código del Producto`,
    '1234567' as `Agencia`,
    'O' as `Tipo de Periodicidad`,
    0.0 as `Tasa vigente`,
    0.0 as `Tasa inicial`,
    '2008-10-31' as `Fecha inicial de tasa`,
    '2140-10-31' as `Fecha fin de tasa`,
    'FI' as `Tipo de tasa`,
    'OT' as `Forma de pago de interés`,
    0.0 as `Tasa de referencia`,
    0.0 as `Porcentaje a pagar por intereses`,
    0.0 as `Porcentaje de comisión`,
    '1' as `Tipo de titularidad`,
    1 as `Número de titulares`,
    'NA' as `Plazo de la Cuenta`,
    '1' as `Condiciones especiales`,
    '' as `Explicación de condiciones especiales`,
    earliest_recorded_at as `Fecha de apertura`,
    end_date as `Fecha de vencimiento`,
    total_collateral_amount_usd as `Monto mínimo`,
    'TODO' as `Código de la cuenta contable`,
    0.0 as `Fondos en compensación`,
    deposit_account_balance_usd as `Fondos restringidos`,
    0.0 as `Transacciones pendientes`,
    '0' as `Negociabilidad del depósito`,
    'BTC' as `Moneda`,
    deposit_account_balance_usd as `Saldo del depósito en la moneda original`,
    latest_recorded_at as `Fecha de la última transacción`,
    0.0 as `Saldo de intereses`,
    deposit_account_balance_usd as `Saldo total`,
    '1' as `Estado`,
    left(replace(upper(deposit_account_id), '-', ''), 20) as `Número de cuenta`,
    last_day(current_date(), month) as `Día de corte`,
    safe_multiply(
        safe_divide(deposit_account_balance_usd, 100000000.0),
        (select last_price_usd from btc_price)
    ) as `Saldo de capital`
from
    final
