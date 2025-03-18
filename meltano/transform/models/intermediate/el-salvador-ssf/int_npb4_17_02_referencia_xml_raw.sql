with loans_and_credit_facilities as (

/* TODO:
    SELECT customer_id, initialized_at, end_date,
        incurrence_interval, accrual_interval, annual_rate,
    loan_id AS reference_id,
    most_recent_interest_payment_timestamp,
    most_recent_principal_payment_timestamp
        AS most_recent_capital_payment_timestamp,
    principal AS loan_amount,
    principal
        + total_interest_incurred
        - total_interest_paid
        - total_principal_paid
        AS remaining_balance,
    principal
        - total_principal_paid AS remaining_capital_balance,
    total_interest_incurred
        - total_interest_paid AS remaining_interest_balance,

    FROM { ref('int_approved_loans') }

    WHERE NOT completed

    UNION ALL
    */

    select
        customer_id,
        initialized_at,
        end_date,
        incurrence_interval,
        accrual_interval,
        annual_rate,
        credit_facility_id as reference_id,
        most_recent_interest_payment_timestamp,
        most_recent_disbursement_payment_timestamp
            as most_recent_capital_payment_timestamp,
        facility as loan_amount,
        total_disbursed
        + total_interest_incurred
        - total_interest_paid
        - total_disbursement_paid as remaining_balance,
        total_disbursed - total_disbursement_paid as remaining_capital_balance,
        total_interest_incurred
        - total_interest_paid as remaining_interest_balance

    from {{ ref('int_approved_credit_facilities') }}

    where not completed

)

select
    left(replace(customer_id, '-', ''), 14) as "nit_deudor",
    '{{ npb4_17_01_tipos_de_cartera(
	'Cartera propia Ley Acceso al Crédito (19)'
	) }}' as "cod_cartera",
    '{{ npb4_17_02_tipos_de_activos_de_riesgo('Préstamos') }}' as "cod_activo",
    left(replace(upper(reference_id), '-', ''), 20) as "num_referencia",
    loan_amount as "monto_referencia",
    remaining_balance as "saldo_referencia",
    remaining_capital_balance as "saldo_vigente_k",
    cast(null as {{ dbt.type_numeric() }}) as "saldo_vencido_k",
    remaining_interest_balance as "saldo_vigente_i",
    cast(null as {{ dbt.type_numeric() }}) as "saldo_vencido_i",
    cast(null as {{ dbt.type_numeric() }}) as "abono_deposito",
    date(initialized_at) as "fecha_otorgamiento",
    date(end_date) as "fecha_vencimiento",
    cast(null as date) as "fecha_castigo",
    '{{ npb4_17_07_estados_de_la_referencia('Vigente') }}' as "estado_credito",
    cast(null as {{ dbt.type_numeric() }}) as "saldo_mora_k",
    cast(null as {{ dbt.type_numeric() }}) as "saldo_mora_i",
    cast(null as {{ dbt.type_int() }}) as "dias_mora_k",
    cast(null as {{ dbt.type_int() }}) as "dias_mora_i",
    cast(null as date) as "fecha_inicio_mora_k",
    cast(null as date) as "fecha_inicio_mora_i",
    case
        when
            accrual_interval = 'end_of_month'
            then '{{ npb4_17_08_formas_de_pago('Mensual') }}'
    end
        as "pago_capital",
    case
        when
            accrual_interval = 'end_of_month'
            then '{{ npb4_17_08_formas_de_pago('Mensual') }}'
    end
        as "pago_interes",
    cast(null as {{ dbt.type_int() }}) as "periodo_gracia_k",
    cast(null as {{ dbt.type_int() }}) as "periodo_gracia_i",
    cast(null as {{ dbt.type_string() }}) as "garante",
    cast(null as {{ dbt.type_string() }}) as "emisión",

    -- join to customer identities's country_of_residence_code?
    9300 as "pais_destino_credito",

    -- join to customer identities's economic_activity_code
    -- or new loan_destination_economic_sector field? required!
    '010101' as "destino",

    '{{ npb4_17_17_monedas('Dólares') }}' as "codigo_moneda",

    -- Interest rate in effect for the reported month.
    cast(annual_rate as {{ dbt.type_numeric() }}) as "tasa_interes",

    -- Nominal interest rate agreed in the contract.
    -- Calculated in relation to the reference rate.
    cast(annual_rate as {{ dbt.type_numeric() }}) as "tasa_contractual",

    -- Reference rate published in the month in which the loan is contracted.
    cast(annual_rate as {{ dbt.type_numeric() }}) as "tasa_referencia",

    -- Specifies the effective rate charged to the client.
    -- Monthly effective rate charged must be calculated
    -- in accordance with Annex 3 of (NBP4-16)
    cast(annual_rate as {{ dbt.type_numeric() }}) as "tasa_efectiva",

    -- "A" for adjustable, "F" for fixed
    'F' as "tipo_tasa_interes",

    '{{ npb4_17_18_tipos_de_prestamos('Crédito decreciente') }}'
        as "tipo_prestamo",
    '{{ npb4_17_21_fuentes_de_recursos('Recursos propios de la entidad') }}'
        as "codigo_recurso",
    cast(null as date) as "ultima_fecha_venc",
    cast(null as {{ dbt.type_numeric() }}) as "dias_prorroga",
    cast(null as {{ dbt.type_numeric() }}) as "monto_desembolsado",
    cast(null as {{ dbt.type_string() }}) as "tipo_credito",
    date(most_recent_interest_payment_timestamp) as "fecha_ultimo_pago_k",
    date(most_recent_capital_payment_timestamp) as "fecha_ultimo_pago_i",
    cast(null as {{ dbt.type_numeric() }}) as "dia_pago_k",
    cast(null as {{ dbt.type_numeric() }}) as "dia_pago_i",
    cast(null as {{ dbt.type_int() }}) as "cuota_mora_k",
    cast(null as {{ dbt.type_int() }}) as "cuota_mora_i",
    cast(null as {{ dbt.type_numeric() }}) as "monto_cuota",

    -- For bank loans, field must be equal to <<114>>
    '114' as "cuenta_contable_k",

    -- For bank loans, field must be equal to <<114>>
    '114' as "cuenta_contable_i",

    cast(null as date) as "fecha_cancelacion",
    cast(null as {{ dbt.type_numeric() }}) as "adelanto_capital",

    -- Corresponds to the reference balance[2.6]
    -- less the proportional value of the guarantees[3.6 / 2.59]
    -- (saldo_referencia - valor_garantia_proporcional)
    cast(null as {{ dbt.type_numeric() }}) as "riesgo_neto",

    cast(null as {{ dbt.type_numeric() }}) as "saldo_seguro",
    cast(null as {{ dbt.type_numeric() }}) as "saldo_costas_procesales",
    cast(null as {{ dbt.type_string() }}) as "tipo_tarjeta_credito",
    cast(null as {{ dbt.type_string() }}) as "clase_tarjeta_credito",
    cast(null as {{ dbt.type_string() }}) as "producto_tarjeta_credito",

    -- Sum of the proportional values ​​of each guarantee[3.6]
    cast(null as {{ dbt.type_numeric() }}) as "valor_garantia_cons",

    cast(null as {{ dbt.type_string() }}) as "municipio_otorgamiento",
    cast(null as {{ dbt.type_numeric() }}) as "reserva_referencia",
    cast(null as {{ dbt.type_string() }}) as "etapa_judicial",
    cast(null as date) as "fecha_demanda",
    cast(null as {{ dbt.type_numeric() }}) as "plazo_credito",
    'SO' as "orden_descuento",
    '{{ npb4_17_03_tipos_de_categorias_de_riesgo('Deudores normales') }}'
        as "categoria_riesgo_ref",
    cast(null as {{ dbt.type_numeric() }}) as "reserva_constituir",
    cast(null as {{ dbt.type_numeric() }}) as "porcentaje_reserva",
    cast(null as {{ dbt.type_numeric() }}) as "pago_cuota",
    cast(null as date) as "fecha_pago",
    cast(null as {{ dbt.type_numeric() }}) as "porcenta_reserva_descon",
    cast(null as {{ dbt.type_numeric() }}) as "porcenta_adiciona_descon",
    cast(null as {{ dbt.type_string() }}) as "depto_destino_credito",
    cast(null as {{ dbt.type_numeric() }}) as "porc_reserva_referencia",
    cast(null as {{ dbt.type_numeric() }}) as "calculo_brecha",
    cast(null as {{ dbt.type_numeric() }}) as "ajuste_brecha",
    cast(null as {{ dbt.type_string() }}) as "programa_asist_cafe",
    cast(null as date) as "fecha_cump_cafe"

from loans_and_credit_facilities
