with disbursals as (
    select
        credit_facility_id,
        disbursal_id,
        recorded_at as disbursal_date,
        disbursal_amount_usd
    from {{ ref('disbursals') }}
    where approved
),

facilities as (

    select *
    from {{ ref('int_approved_credit_facilities') }}
)

select
    left(replace(f.customer_id, '-', ''), 14) as `nit_deudor`,
    '{{ npb4_17_01_tipos_de_cartera(
	'Cartera propia Ley Acceso al Crédito (19)'
	) }}' as `cod_cartera`,
    '{{ npb4_17_02_tipos_de_activos_de_riesgo('Préstamos') }}' as `cod_activo`,
    left(replace(upper(d.disbursal_id), '-', ''), 20) as `num_referencia`,
    d.disbursal_amount_usd as `monto_referencia`,
    d.disbursal_amount_usd as `saldo_referencia`,
    d.disbursal_amount_usd as `saldo_vigente_k`,
    cast(null as numeric) as `saldo_vencido_k`,
    cast(null as numeric) as `saldo_vigente_i`,
    cast(null as numeric) as `saldo_vencido_i`,
    cast(null as numeric) as `abono_deposito`,
    date(f.initialized_at) as `fecha_otorgamiento`,
    date(f.end_date) as `fecha_vencimiento`,
    cast(null as date) as `fecha_castigo`,
    '{{ npb4_17_07_estados_de_la_referencia('Vigente') }}' as `estado_credito`,
    cast(null as numeric) as `saldo_mora_k`,
    cast(null as numeric) as `saldo_mora_i`,
    cast(null as int64) as `dias_mora_k`,
    cast(null as int64) as `dias_mora_i`,
    cast(null as date) as `fecha_inicio_mora_k`,
    cast(null as date) as `fecha_inicio_mora_i`,
    case
        when
            f.accrual_cycle_interval = 'end_of_month'
            then '{{ npb4_17_08_formas_de_pago('Mensual') }}'
    end
        as `pago_capital`,
    case
        when
            f.accrual_cycle_interval = 'end_of_month'
            then '{{ npb4_17_08_formas_de_pago('Mensual') }}'
    end
        as `pago_interes`,
    cast(null as int64) as `periodo_gracia_k`,
    cast(null as int64) as `periodo_gracia_i`,
    cast(null as string) as `garante`,
    cast(null as string) as `emisión`,

    9300 as `pais_destino_credito`,

    '010101' as `destino`,

    '{{ npb4_17_17_monedas('Dólares') }}' as `codigo_moneda`,

    cast(f.annual_rate as numeric) as `tasa_interes`,
    cast(f.annual_rate as numeric) as `tasa_contractual`,
    cast(f.annual_rate as numeric) as `tasa_referencia`,
    cast(f.annual_rate as numeric) as `tasa_efectiva`,

    'F' as `tipo_tasa_interes`,

    '{{ npb4_17_18_tipos_de_prestamos('Crédito decreciente') }}'
        as `tipo_prestamo`,
    '{{ npb4_17_21_fuentes_de_recursos('Recursos propios de la entidad') }}'
        as `codigo_recurso`,
    cast(null as date) as `ultima_fecha_venc`,
    cast(null as numeric) as `dias_prorroga`,
    d.disbursal_amount_usd as `monto_desembolsado`,
    cast(null as string) as `tipo_credito`,
    cast(null as date) as `fecha_ultimo_pago_k`,
    cast(null as date) as `fecha_ultimo_pago_i`,
    cast(null as numeric) as `dia_pago_k`,
    cast(null as numeric) as `dia_pago_i`,
    cast(null as int64) as `cuota_mora_k`,
    cast(null as int64) as `cuota_mora_i`,
    cast(null as numeric) as `monto_cuota`,

    '114' as `cuenta_contable_k`,
    '114' as `cuenta_contable_i`,

    cast(null as date) as `fecha_cancelacion`,
    cast(null as numeric) as `adelanto_capital`,
    cast(null as numeric) as `riesgo_neto`,
    cast(null as numeric) as `saldo_seguro`,
    cast(null as numeric) as `saldo_costas_procesales`,
    cast(null as string) as `tipo_tarjeta_credito`,
    cast(null as string) as `clase_tarjeta_credito`,
    cast(null as string) as `producto_tarjeta_credito`,
    cast(null as numeric) as `valor_garantia_cons`,
    cast(null as string) as `municipio_otorgamiento`,
    cast(null as numeric) as `reserva_referencia`,
    cast(null as string) as `etapa_judicial`,
    cast(null as date) as `fecha_demanda`,
    cast(null as numeric) as `plazo_credito`,
    'SO' as `orden_descuento`,
    '{{ npb4_17_03_tipos_de_categorias_de_riesgo('Deudores normales') }}'
        as `categoria_riesgo_ref`,
    cast(null as numeric) as `reserva_constituir`,
    cast(null as numeric) as `porcentaje_reserva`,
    cast(null as numeric) as `pago_cuota`,
    cast(null as date) as `fecha_pago`,
    cast(null as numeric) as `porcenta_reserva_descon`,
    cast(null as numeric) as `porcenta_adiciona_descon`,
    cast(null as string) as `depto_destino_credito`,
    cast(null as numeric) as `porc_reserva_referencia`,
    cast(null as numeric) as `calculo_brecha`,
    cast(null as numeric) as `ajuste_brecha`,
    cast(null as string) as `programa_asist_cafe`,
    cast(null as date) as `fecha_cump_cafe`

from disbursals d
join facilities f on d.credit_facility_id = f.credit_facility_id
