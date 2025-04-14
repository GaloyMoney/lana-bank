{{ config(materialized='table') }}

select
    left(`Código del Producto`, 4) as `Código del Producto`,
    left(`Agencia`, 7) as `Agencia`,
    left(`Tipo de Periodicidad`, 1) as `Tipo de Periodicidad`,
    cast(round(`Tasa vigente`, 2) as string) as `Tasa vigente`,
    cast(round(`Tasa inicial`, 2) as string) as `Tasa inicial`,
    format_date('%Y%m%d', cast(`Fecha inicial de tasa` as date)) as `Fecha inicial de tasa`,
    format_date('%Y%m%d', cast(`Fecha fin de tasa` as date)) as `Fecha fin de tasa`,
    left(`Tipo de tasa`, 2) as `Tipo de tasa`,
    left(`Forma de pago de interés`, 2) as `Forma de pago de interés`,
    cast(round(`Tasa de referencia`, 2) as string) as `Tasa de referencia`,
    cast(round(`Porcentaje a pagar por intereses`, 2) as string)
        as `Porcentaje a pagar por intereses`,
    cast(round(`Porcentaje de comisión`, 2) as string) as `Porcentaje de comisión`,
    left(`Tipo de titularidad`, 1) as `Tipo de titularidad`,
    cast(`Número de titulares` as string) as `Número de titulares`,
    left(`Plazo de la Cuenta`, 8) as `Plazo de la Cuenta`,
    left(`Condiciones especiales`, 1) as `Condiciones especiales`,
    left(`Explicación de condiciones especiales`, 100) as `Explicación de condiciones especiales`,
    format_date('%Y%m%d', cast(`Fecha de apertura` as date)) as `Fecha de apertura`,
    format_date('%Y%m%d', cast(`Fecha de vencimiento` as date)) as `Fecha de vencimiento`,
    cast(round(`Monto mínimo`, 2) as string) as `Monto mínimo`,
    left(`Código de la cuenta contable`, 20) as `Código de la cuenta contable`,
    cast(round(`Fondos en compensación`, 2) as string) as `Fondos en compensación`,
    cast(round(`Fondos restringidos`, 2) as string) as `Fondos restringidos`,
    cast(round(`Transacciones pendientes`, 2) as string) as `Transacciones pendientes`,
    left(`Negociabilidad del depósito`, 1) as `Negociabilidad del depósito`,
    left(`Moneda`, 3) as `Moneda`,
    cast(round(`Saldo del depósito en la moneda original`, 2) as string)
        as `Saldo del depósito en la moneda original`,
    format_date('%Y%m%d', cast(`Fecha de la última transacción` as date))
        as `Fecha de la última transacción`,
    cast(round(`Saldo de intereses`, 2) as string) as `Saldo de intereses`,
    cast(round(`Saldo total`, 2) as string) as `Saldo total`,
    left(`Estado`, 1) as `Estado`,
    left(`Número de cuenta`, 20) as `Número de cuenta`,
    format_date('%Y%m%d', cast(`Día de corte` as date)) as `Día de corte`,
    cast(round(`Saldo de capital`, 2) as string) as `Saldo de capital`
from
    {{ ref('int_nrsf_03_02_depósitos') }}
