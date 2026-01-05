with credit_facility_loans as (
    select
        credit_facility_id,
        customer_id,
        disbursal_approved_recorded_at,
        disbursal_end_date,
        duration_value,
        duration_type,
        accrual_interval,
        accrual_cycle_interval,
        annual_rate,
        disbursal_id as reference_id,
        most_recent_interest_payment_timestamp,
        most_recent_disbursal_payment_timestamp as most_recent_capital_payment_timestamp,
        collateral_amount_usd,
        total_disbursed_usd as loan_amount_usd,
        total_disbursed_usd
        + interest_incurred_usd
        - interest_paid_usd
        - disbursal_paid_usd
            as remaining_balance_usd,
        total_disbursed_usd
        - disbursal_paid_usd
            as remaining_capital_balance_usd,
        interest_incurred_usd
        - interest_paid_usd
            as remaining_interest_balance_usd

    from {{ ref('int_approved_credit_facility_loans') }}

    where not matured
),

capital_overdue as (
    select
        credit_facility_id,
        min(overdue_date) as capital_overdue_date,
        max(overdue_days) as capital_overdue_days
    from {{ ref('int_core_obligation_events_rollup') }}
    where overdue_days > 0 and obligation_type = 'Disbursal'
    group by credit_facility_id
),

interest_overdue as (
    select
        credit_facility_id,
        min(overdue_date) as interest_overdue_date,
        max(overdue_days) as interest_overdue_days
    from {{ ref('int_core_obligation_events_rollup') }}
    where overdue_days > 0 and obligation_type = 'Interest'
    group by credit_facility_id
),

loans_with_overdue_days as (
    select
        cfl.*,
        capital_overdue_date,
        interest_overdue_date,
        coalesce(capital_overdue_days, 0) as capital_overdue_days,
        coalesce(interest_overdue_days, 0) as interest_overdue_days,
        greatest(
            coalesce(capital_overdue_days, 0),
            coalesce(interest_overdue_days, 0)
        ) as payment_overdue_days

    from credit_facility_loans as cfl
    left join capital_overdue using (credit_facility_id)
    left join interest_overdue using (credit_facility_id)
),

risk_category as (
    select
        od.*,
        r.category as risk_category_ref,
        r.reserve_percentage,
        remaining_balance_usd - collateral_amount_usd as net_risk
    from loans_with_overdue_days as od
    left join
        {{ ref('static_ncb_022_porcentaje_reservas_saneamiento') }} as r
        on
            od.payment_overdue_days between r.consumer_calendar_ge_days
            and r.consumer_calendar_le_days
),

final as (
    select
        *,
        reserve_percentage * greatest(0, net_risk) as reserve
    from risk_category
)

select
    credit_facility_public_ids.id as credit_facility_id,
    customer_public_ids.id as customer_id,
    disbursement_public_ids.id as disbursal_id,
    disbursement_public_ids.id as reference_id,
    customer_public_ids.id as `nit_deudor`,
    '{{ npb4_17_01_tipos_de_cartera("Cartera propia Ley Acceso al Crédito (19)") }}'
        as `cod_cartera`,
    '{{ npb4_17_02_tipos_de_activos_de_riesgo("Préstamos") }}' as `cod_activo`,
    disbursement_public_ids.id as `num_referencia`,
    loan_amount_usd as `monto_referencia`,
    remaining_balance_usd,
    remaining_balance_usd as `saldo_referencia`,
    remaining_capital_balance_usd as `saldo_vigente_k`,
    cast(null as numeric) as `saldo_vencido_k`,
    remaining_interest_balance_usd as `saldo_vigente_i`,
    cast(null as numeric) as `saldo_vencido_i`,
    cast(null as numeric) as `abono_deposito`,
    date(disbursal_approved_recorded_at) as `fecha_otorgamiento`,
    date(disbursal_end_date) as `fecha_vencimiento`,
    cast(null as date) as `fecha_castigo`,
    '{{ npb4_17_07_estados_de_la_referencia("Vigente") }}' as `estado_credito`,
    cast(null as numeric) as `saldo_mora_k`,
    cast(null as numeric) as `saldo_mora_i`,
    capital_overdue_days as `dias_mora_k`,
    interest_overdue_days as `dias_mora_i`,
    capital_overdue_date as `fecha_inicio_mora_k`,
    interest_overdue_date as `fecha_inicio_mora_i`,
    case
        when
            accrual_cycle_interval = 'end_of_month'
            then '{{ npb4_17_08_formas_de_pago("Anual") }}'
    end
        as `pago_capital`,
    case
        when
            accrual_cycle_interval = 'end_of_month'
            then '{{ npb4_17_08_formas_de_pago("Mensual") }}'
    end
        as `pago_interes`,
    cast(null as int64) as `periodo_gracia_k`,
    cast(null as int64) as `periodo_gracia_i`,
    cast(null as string) as `garante`,
    cast(null as string) as `emisión`,

    -- join to customer identities's country_of_residence_code?
    9300 as `pais_destino_credito`,

    -- join to customer identities's economic_activity_code
    -- or new loan_destination_economic_sector field? required!
    '010101' as `destino`,

    '{{ npb4_17_17_monedas("Dólares") }}' as `codigo_moneda`,

    -- Interest rate in effect for the reported month.
    cast(annual_rate as numeric) as `tasa_interes`,

    -- Nominal interest rate agreed in the contract.
    -- Calculated in relation to the reference rate.
    cast(annual_rate as numeric) as `tasa_contractual`,

    -- Reference rate published in the month in which the loan is contracted.
    cast(annual_rate as numeric) as `tasa_referencia`,

    -- Specifies the effective rate charged to the client.
    -- Monthly effective rate charged must be calculated
    -- in accordance with Annex 3 of (NBP4-16)
    cast(annual_rate as numeric) as `tasa_efectiva`,

    -- "A" for adjustable, "F" for fixed
    'F' as `tipo_tasa_interes`,

    '{{ npb4_17_18_tipos_de_prestamos("Crédito decreciente") }}'
        as `tipo_prestamo`,
    '{{ npb4_17_21_fuentes_de_recursos("Recursos propios de la entidad") }}'
        as `codigo_recurso`,
    cast(null as date) as `ultima_fecha_venc`,
    cast(null as numeric) as `dias_prorroga`,
    cast(null as numeric) as `monto_desembolsado`,
    cast(null as string) as `tipo_credito`,
    date(most_recent_interest_payment_timestamp) as `fecha_ultimo_pago_k`,
    date(most_recent_capital_payment_timestamp) as `fecha_ultimo_pago_i`,
    extract(day from most_recent_interest_payment_timestamp) as `dia_pago_k`,
    extract(day from most_recent_capital_payment_timestamp) as `dia_pago_i`,
    cast(null as int64) as `cuota_mora_k`,
    cast(null as int64) as `cuota_mora_i`,
    cast(null as numeric) as `monto_cuota`,

    -- For bank loans, field must be equal to <<114>>
    '114' as `cuenta_contable_k`,

    -- For bank loans, field must be equal to <<114>>
    '114' as `cuenta_contable_i`,

    cast(null as date) as `fecha_cancelacion`,
    cast(null as numeric) as `adelanto_capital`,

    -- Corresponds to the reference balance[2.6]
    -- less the proportional value of the guarantees[3.6 / 2.59]
    -- (saldo_referencia - valor_garantia_proporcional)
    net_risk,
    net_risk as `riesgo_neto`,

    cast(null as numeric) as `saldo_seguro`,
    cast(null as numeric) as `saldo_costas_procesales`,
    cast(null as string) as `tipo_tarjeta_credito`,
    cast(null as string) as `clase_tarjeta_credito`,
    cast(null as string) as `producto_tarjeta_credito`,

    -- Sum of the proportional values ​​of each guarantee[3.6]
    collateral_amount_usd,
    collateral_amount_usd as `valor_garantia_cons`,

    cast(null as string) as `distrito_otorgamiento`,
    reserve,
    reserve as `reserva_referencia`,
    cast(null as string) as `etapa_judicial`,
    cast(null as date) as `fecha_demanda`,
    duration_value as `plazo_credito`,
    'SO' as `orden_descuento`,
    risk_category_ref,
    risk_category_ref as `categoria_riesgo_ref`,
    cast(null as numeric) as `reserva_constituir`,
    cast(null as numeric) as `porcentaje_reserva`,
    cast(null as numeric) as `pago_cuota`,
    cast(null as date) as `fecha_pago`,
    cast(null as numeric) as `porcenta_reserva_descon`,
    cast(null as numeric) as `porcenta_adiciona_descon`,
    cast(null as string) as `depto_destino_credito`,
    reserve_percentage,
    reserve_percentage as `porc_reserva_referencia`,
    cast(null as numeric) as `calculo_brecha`,
    cast(null as numeric) as `ajuste_brecha`,
    cast(null as string) as `programa_asist_cafe`,
    cast(null as date) as `fecha_cump_cafe`

from final
left join
    {{ ref('stg_core_public_ids') }} as credit_facility_public_ids
    on credit_facility_id = credit_facility_public_ids.target_id
left join
    {{ ref('stg_core_public_ids') }} as disbursement_public_ids
    on reference_id = disbursement_public_ids.target_id
left join
    {{ ref('stg_core_public_ids') }} as customer_public_ids
    on customer_id = customer_public_ids.target_id
