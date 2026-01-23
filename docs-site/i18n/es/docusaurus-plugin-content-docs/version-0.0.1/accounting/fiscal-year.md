---
id: fiscal-year
title: Año Fiscal
sidebar_position: 3
---

# AñoFiscal

## Supuestos Importantes

- Las cuentas del `EstadoDeResultados` operan solo en USD.

## Inicialización

No se pueden registrar transacciones hasta que la primera entidad `AñoFiscal` haya sido inicializada. Una actualización de metadatos del conjunto de cuentas realizada al conjunto de cuentas raíz del `Plan` al que se relaciona el `AñoFiscal` permite que los controles de velocidad sean satisfechos.

Hay 2 formas de inicializar el primer `AñoFiscal`:

- En el inicio inicial si `accounting_init` tiene una `chart_of_accounts_opening_date` establecida en YAML.
- A través de mutación GraphQL.

## Cierre de Meses de un AñoFiscal

Los cierres mensuales bloquean todo el libro mayor contra transacciones con una `effective_date` anterior a `month_closed_as_of`. Para ejecutar este comando de entidad, la precondición de que el mes haya pasado (según `crate::time::now()`) debe ser satisfecha. El comando aplica al mes más antiguo no cerrado del `AñoFiscal`.

## Cierre del AñoFiscal

Si el último mes de un `AñoFiscal` ha sido cerrado, el ciclo de vida del `AñoFiscal` puede ser completado. Esto registra una transacción en el libro mayor, con una `effective_date` establecida al `closed_as_of` del `AñoFiscal`.

La significancia contable de esta transacción es transferir el ingreso neto del `AñoFiscal` del `EstadoDeResultados` al `BalanceGeneral`.

## Abrir el siguiente AñoFiscal

Requerido como una acción explícita.
