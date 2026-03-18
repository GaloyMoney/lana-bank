---
id: ledger
title: Libro mayor
sidebar_position: 8
---

# Descripción General del Libro Mayor

Este documento describe los conjuntos de cuentas creados por el módulo de crédito durante la inicialización, su contexto contable y las plantillas de transacciones que estructuran el flujo de fondos entre cuentas.

## Conjuntos de Cuentas Ómnibus

Todos los conjuntos de cuentas ómnibus están fuera de balance y contienen una única cuenta compartida.

### Línea de Crédito

```
Ref: credit-facility-omnibus-account-set
Name: Conjunto de Cuentas Ómnibus de Líneas de Crédito
Purpose: Rastrea los compromisos totales de líneas de crédito en todas las líneas
Category: Fuera de Balance
Normal Balance: Débito
Account Creation: Compartida (1 cuenta: credit-facility-omnibus-account)
```

### Garantía

```
Ref: credit-collateral-omnibus-account-set
Name: Conjunto de Cuentas Ómnibus de Garantías de Crédito
Purpose: Rastrea el total de garantías depositadas en todas las líneas
Category: Fuera de Balance
Normal Balance: Débito
Account Creation: Compartida (1 cuenta: credit-collateral-omnibus-account)
────────────────────────────────────────
Ref: credit-facility-liquidation-proceeds-omnibus-account-set
Name: Conjunto de Cuentas Ómnibus de Ingresos por Liquidación de Líneas de Crédito
Purpose: Rastrea el total de ingresos por liquidación recibidos en todas las líneas
Category: Fuera de Balance
Normal Balance: Débito
Account Creation: Compartida (1 cuenta: credit-facility-liquidation-proceeds-omnibus-account)
```

### Intereses

```
Ref: credit-interest-added-to-obligations-omnibus-account-set
Name: Conjunto de Cuentas Ómnibus de Intereses Agregados a Obligaciones de Crédito
Purpose: Rastrea el total de intereses registrados agregados a las obligaciones del prestatario
Category: Fuera de Balance
Normal Balance: Débito
Account Creation: Compartida (1 cuenta: credit-interest-added-to-obligations-omnibus-account)
```

### Pagos

```
Ref: credit-payments-made-omnibus-account-set
Name: Conjunto de Cuentas Ómnibus de Pagos Realizados de Crédito
Purpose: Rastrea el total de pagos recibidos en todas las líneas
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Compartida (1 cuenta: credit-payments-made-omnibus-account)
```

## Conjuntos de Cuentas Resumen

Todos los conjuntos de cuentas resumen agregan saldos/transacciones de cuentas que se crean por línea de crédito.

### Línea de Crédito

```
Ref: credit-facility-remaining-account-set
Name: Conjunto de Cuentas de Saldo Restante de Línea de Crédito
Purpose: Rastrea el saldo de la línea no utilizado
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Por línea
────────────────────────────────────────
Ref: credit-uncovered-outstanding-account-set
Name: Conjunto de Cuentas de Saldo Pendiente No Cubierto de Crédito
Purpose: Rastrea el monto pendiente aún no cubierto por un pago no aplicado
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Por línea
```

### Garantía

```
Ref: credit-collateral-account-set
Name: Conjunto de Cuentas de Garantía de Crédito
Purpose: Rastrea la garantía comprometida a la facilidad
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Por facilidad
────────────────────────────────────────
Ref: credit-facility-collateral-in-liquidation-account-set
Name: Conjunto de Cuentas de Garantía en Liquidación de Facilidad de Crédito
Purpose: Rastrea la garantía en liquidación activa
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Por facilidad
────────────────────────────────────────
Ref: credit-facility-liquidated-collateral-account-set
Name: Conjunto de Cuentas de Garantía Liquidada de Facilidad de Crédito
Purpose: Rastrea la garantía que ha sido liquidada
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Por facilidad
────────────────────────────────────────
Ref: credit-facility-proceeds-from-liquidation-account-set
Name: Conjunto de Cuentas de Productos de Liquidación de Facilidad de Crédito
Purpose: Rastrea los ingresos recibidos de la liquidación de garantías
Category: Fuera de Balance
Normal Balance: Crédito
Account Creation: Por facilidad
```

### Cuentas por Cobrar Desembolsadas a Corto Plazo

Todos los conjuntos en este grupo son de categoría Activo, saldo normal Débito, por facilidad.

Patrón de referencia: `short-term-credit-{type}-disbursed-receivable-account-set`

Propósito: Rastrea el principal adeudado en facilidades a corto plazo, por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Cuentas por Cobrar Desembolsadas a Largo Plazo

Todos los conjuntos en este grupo son de categoría Activo, saldo normal Débito, por facilidad.

Patrón de referencia: `long-term-credit-{type}-disbursed-receivable-account-set`

Propósito: Rastrea el principal adeudado en facilidades a largo plazo, por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Cuentas por Cobrar Desembolsadas Vencidas

Todos los conjuntos en este grupo son de categoría Activo, saldo normal Débito, por facilidad.

Patrón de referencia: `overdue-credit-{type}-disbursed-receivable-account-set`

Propósito: Rastrea el principal vencido, por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Cuentas por Cobrar en Mora

```
Ref: credit-disbursed-defaulted-account-set
Name: Conjunto de Cuentas de Principal Desembolsado en Mora
Purpose: Rastrea el principal en mora
Category: Activo
Normal Balance: Débito
Account Creation: Por facilidad
────────────────────────────────────────
Ref: credit-interest-defaulted-account-set
Name: Conjunto de Cuentas de Interés en Mora
Purpose: Rastrea el interés en mora
Category: Activo
Normal Balance: Débito
Account Creation: Por facilidad
```

### Intereses por Cobrar a Corto Plazo

Todos los conjuntos en este grupo son categoría Activo, saldo normal Débito, por facilidad.

Patrón de referencia: `short-term-credit-{type}-interest-receivable-account-set`

Propósito: Rastrea los intereses adeudados en facilidades a corto plazo, por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Intereses por Cobrar a Largo Plazo

Todos los conjuntos en este grupo son categoría Activo, saldo normal Débito, por facilidad.

Patrón de referencia: `long-term-credit-{type}-interest-receivable-account-set`

Propósito: Rastrea los intereses adeudados en facilidades a largo plazo, por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Ingresos

```
Ref: credit-interest-income-account-set
Name: Conjunto de Cuentas de Ingresos por Intereses de Crédito
Purpose: Ingresos por intereses reconocidos
Category: Ingresos
Normal Balance: Crédito
Account Creation: Por facilidad
────────────────────────────────────────
Ref: credit-fee-income-account-set
Name: Conjunto de Cuentas de Ingresos por Comisiones de Crédito
Purpose: Ingresos por comisiones reconocidos (comisiones de estructuración)
Category: Ingresos
Normal Balance: Crédito
Account Creation: Por facilidad
```

### Retención de Pagos

```
Ref: credit-payment-holding-account-set
Name: Conjunto de Cuentas de Retención de Pagos de Crédito
Purpose: Retiene temporalmente los pagos que están esperando ser aplicados a las obligaciones
Category: Activo
Normal Balance: Crédito
Account Creation: Por facilidad
```

## Plantillas de Transacciones

Las columnas a la derecha del Código de Plantilla representan los conjuntos de cuentas involucrados en la transacción. Un valor de celda indica que la plantilla debita (DR) o acredita (CR) una cuenta de ese conjunto de cuentas, en la capa Liquidada o Pendiente. Las celdas vacías significan que la plantilla no involucra ese conjunto de cuentas.

### Facilidad

```
┌─────────────────────────────────────┬──────────────────────┬──────────────────────┐
│ Código de Plantilla                │ Ómnibus de Facilidad │ Remanente Facilidad  │
├─────────────────────────────────────┼──────────────────────┼──────────────────────┤
│ CREATE_CREDIT_FACILITY_PROPOSAL     │ DR (Pendiente)       │ CR (Pendiente)       │
├─────────────────────────────────────┼──────────────────────┼──────────────────────┤
│ ACTIVATE_CREDIT_FACILITY            │ CR (Pendiente)       │ DR (Pendiente)       │
│                                     │ DR (Liquidado)       │ CR (Liquidado)       │
└─────────────────────────────────────┴──────────────────────┴──────────────────────┘
```

### Desembolsos

```
┌──────────────────────────┬─────────────┬─────────────┬─────────────┬──────────────────┐
│ Template Code            │ Facility    │ Uncovered   │ Disbursed   │ Deposit Omnibus  │
│                          │ Remaining   │ Outstanding │ Receivable  │ (external)       │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ INITIAL_DISBURSAL        │ DR (Settled)│ CR (Settled)│ DR (Settled)│ CR (Settled)     │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ INITIATE_CREDIT_FACILITY │ DR (Settled)│ CR (Settled)│             │                  │
│ _DISBURSAL               │ CR (Pending)│ DR (Pending)│             │                  │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ CONFIRM_DISBURSAL        │ DR (Pending)│ CR (Pending)│ DR (Settled)│ CR (Settled)     │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ CANCEL_DISBURSAL         │ DR (Pending)│ CR (Pending)│             │                  │
│                          │ CR (Settled)│ DR (Settled)│             │                  │
└──────────────────────────┴─────────────┴─────────────┴─────────────┴──────────────────┘
```

### Intereses

```
┌────────────────────────────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│ Template Code                      │ Interest    │ Interest    │ Int. Added  │ Uncovered   │
│                                    │ Receivable  │ Income      │ to Oblig.   │ Outstanding │
│                                    │             │             │ Omnibus     │             │
├────────────────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│ CREDIT_FACILITY_ACCRUE_INTEREST    │ DR (Pending)│ CR (Pending)│             │             │
├────────────────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│ CREDIT_FACILITY_POST_ACCRUED_      │ CR (Pending)│ DR (Pending)│             │             │
│ INTEREST                           │ DR (Settled)│ CR (Settled)│ DR (Settled)│ CR (Settled)│
└────────────────────────────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
```

### Comisiones

```
┌─────────────────────┬────────────────────┬──────────────┐
│ Template Code       │ Disbursed          │ Fee Income   │
│                     │ Receivable         │              │
├─────────────────────┼────────────────────┼──────────────┤
│ ADD_STRUCTURING_FEE │ DR (Settled)       │ CR (Settled) │
└─────────────────────┴────────────────────┴──────────────┘
```
