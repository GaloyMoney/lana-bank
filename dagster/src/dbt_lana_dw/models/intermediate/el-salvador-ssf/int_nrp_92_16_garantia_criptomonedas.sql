select
    disbursement_public_ids.id as `num_referencia`,

    '01' as `cod_cartera`,

    '{{ npb4_17_02_tipos_de_activos_de_riesgo("Préstamos") }}' as `cod_activo`,

    disbursement_public_ids.id as `identificacion_garantia`,

    collateral_amount_btc as `num_criptomonedas`,

    -- USD value at the time collateral was first deposited
    collateral_amount_btc * initial_price_usd_per_btc as `valor_contractual`,

    -- Current USD market value (latest BTC price * BTC amount)
    collateral_amount_usd as `valor_mercado`,

    date(latest_price_timestamp) as `fecha_valuacion_mercado`,

    format_timestamp('%H:%M:%S', latest_price_timestamp) as `hora_valuacion_mercado`,

    latest_price_usd_per_btc as `tasas_conversion`,

    -- Proportional collateral amount covering this disbursal
    collateral_amount_usd as `monto_garantizado`,

    date(most_recent_collateral_deposit_at) as `fecha_otorgamiento`

from {{ ref("int_approved_credit_facility_loans") }}
left join
    {{ ref("stg_core_public_ids") }} as disbursement_public_ids
    on disbursal_id = disbursement_public_ids.target_id

where not matured
