---
id: ledger
title: Libro Mayor
sidebar_position: 8
---

# Resumen del Libro Mayor

Este documento describe los conjuntos de cuentas creados por el módulo de crédito durante la inicialización, su contexto contable y las plantillas de transacción que estructuran el flujo de fondos entre cuentas.

## Conjuntos de Cuentas Ómnibus

Todos los conjuntos de cuentas ómnibus son fuera de balance y contienen una sola cuenta compartida.

### Facilidad

```
Referencia: credit-facility-omnibus-account-set
Nombre: Conjunto de Cuentas Ómnibus de Facilidad de Crédito
Propósito: Rastrea el compromiso total de líneas de crédito en todas las facilidades
Categoría: Fuera de balance
Balance normal: Débito
Creación de cuenta: Compartida (1 cuenta: credit-facility-omnibus-account)
```

### Colateral

```
Referencia: credit-collateral-omnibus-account-set
Nombre: Conjunto de Cuentas Ómnibus de Colateral de Crédito
Propósito: Rastrea el colateral total depositado en todas las facilidades
Categoría: Fuera de balance
Balance normal: Débito
Creación de cuenta: Compartida (1 cuenta: credit-collateral-omnibus-account)
────────────────────────────────────────
Referencia: credit-facility-liquidation-proceeds-omnibus-account-set
Nombre: Conjunto de Cuentas Ómnibus de Producto de Liquidación de Facilidades de Crédito
Propósito: Rastrea los productos totales de liquidación recibidos en todas las facilidades
Categoría: Fuera de balance
Balance normal: Débito
Creación de cuenta: Compartida (1 cuenta: credit-facility-liquidation-proceeds-omnibus-account)
```

### Interés

```
Referencia: credit-interest-added-to-obligations-omnibus-account-set
Nombre: Conjunto de Cuentas Ómnibus de Interés Añadido a Obligaciones
Propósito: Rastrea el interés total contabilizado y añadido a las obligaciones del prestatario
Categoría: Fuera de balance
Balance normal: Débito
Creación de cuenta: Compartida (1 cuenta: credit-interest-added-to-obligations-omnibus-account)
```

### Pagos

```
Referencia: credit-payments-made-omnibus-account-set
Nombre: Conjunto de Cuentas Ómnibus de Pagos de Crédito Recibidos
Propósito: Rastrea el total de pagos recibidos en todas las facilidades
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Compartida (1 cuenta: credit-payments-made-omnibus-account)
```

## Conjuntos de Cuentas Resumen

Todos los conjuntos de cuentas resumen agregan saldos y transacciones de cuentas creadas por facilidad de crédito.

### Facilidad

```
Referencia: credit-facility-remaining-account-set
Nombre: Conjunto de Cuentas de Remanente de Facilidad de Crédito
Propósito: Rastrea el saldo no dispuesto de la facilidad
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Por facilidad
────────────────────────────────────────
Referencia: credit-uncovered-outstanding-account-set
Nombre: Conjunto de Cuentas de Pendiente sin Cobertura de Crédito
Propósito: Rastrea el monto pendiente aún no cubierto por un pago no aplicado
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Por facilidad
```

### Colateral

```
Referencia: credit-collateral-account-set
Nombre: Conjunto de Cuentas de Colateral de Crédito
Propósito: Rastrea el colateral comprometido a la facilidad
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Por facilidad
────────────────────────────────────────
Referencia: credit-facility-collateral-in-liquidation-account-set
Nombre: Conjunto de Cuentas de Colateral en Liquidación de Facilidad de Crédito
Propósito: Rastrea el colateral en liquidación activa
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Por facilidad
────────────────────────────────────────
Referencia: credit-facility-liquidated-collateral-account-set
Nombre: Conjunto de Cuentas de Colateral Liquidado de Facilidad de Crédito
Propósito: Rastrea el colateral que ya fue liquidado
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Por facilidad
────────────────────────────────────────
Referencia: credit-facility-proceeds-from-liquidation-account-set
Nombre: Conjunto de Cuentas de Producto por Liquidación de Facilidad de Crédito
Propósito: Rastrea los productos recibidos por liquidación de colateral
Categoría: Fuera de balance
Balance normal: Crédito
Creación de cuenta: Por facilidad
```

### Cartera Desembolsada de Corto Plazo

Todos los conjuntos en este grupo pertenecen a la categoría de Activos, tienen balance normal débito y se crean por facilidad.

Patrón de referencia: `short-term-credit-{type}-disbursed-receivable-account-set`

Propósito: rastrear el principal adeudado en facilidades de corto plazo por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Cartera Desembolsada de Largo Plazo

Todos los conjuntos en este grupo pertenecen a la categoría de Activos, tienen balance normal débito y se crean por facilidad.

Patrón de referencia: `long-term-credit-{type}-disbursed-receivable-account-set`

Propósito: rastrear el principal adeudado en facilidades de largo plazo por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Cartera Desembolsada en Mora

Todos los conjuntos en este grupo pertenecen a la categoría de Activos, tienen balance normal débito y se crean por facilidad.

Patrón de referencia: `overdue-credit-{type}-disbursed-receivable-account-set`

Propósito: rastrear principal vencido por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Cartera en Incumplimiento

```
Referencia: credit-disbursed-defaulted-account-set
Nombre: Conjunto de Cuentas de Cartera Desembolsada en Incumplimiento
Propósito: Rastrea principal en incumplimiento
Categoría: Activo
Balance normal: Débito
Creación de cuenta: Por facilidad
────────────────────────────────────────
Referencia: credit-interest-defaulted-account-set
Nombre: Conjunto de Cuentas de Interés en Incumplimiento
Propósito: Rastrea interés en incumplimiento
Categoría: Activo
Balance normal: Débito
Creación de cuenta: Por facilidad
```

### Interés por Cobrar de Corto Plazo

Todos los conjuntos en este grupo pertenecen a la categoría de Activos, tienen balance normal débito y se crean por facilidad.

Patrón de referencia: `short-term-credit-{type}-interest-receivable-account-set`

Propósito: rastrear intereses adeudados en facilidades de corto plazo por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Interés por Cobrar de Largo Plazo

Todos los conjuntos en este grupo pertenecen a la categoría de Activos, tienen balance normal débito y se crean por facilidad.

Patrón de referencia: `long-term-credit-{type}-interest-receivable-account-set`

Propósito: rastrear intereses adeudados en facilidades de largo plazo por tipo de cliente.

Donde `{type}` es uno de: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Ingresos

```
Referencia: credit-interest-income-account-set
Nombre: Conjunto de Cuentas de Ingreso por Interés de Crédito
Propósito: Ingreso por interés reconocido
Categoría: Ingresos
Balance normal: Crédito
Creación de cuenta: Por facilidad
────────────────────────────────────────
Referencia: credit-fee-income-account-set
Nombre: Conjunto de Cuentas de Ingreso por Comisiones de Crédito
Propósito: Ingreso por comisiones reconocido (comisiones de estructuración)
Categoría: Ingresos
Balance normal: Crédito
Creación de cuenta: Por facilidad
```

### Cuenta Puente de Pagos

```
Referencia: credit-payment-holding-account-set
Nombre: Conjunto de Cuentas Puente de Pagos de Crédito
Propósito: Retiene temporalmente pagos que esperan aplicación a obligaciones
Categoría: Activo
Balance normal: Crédito
Creación de cuenta: Por facilidad
```

## Plantillas de Transacción

Las columnas a la derecha de `Código de plantilla` representan los conjuntos de cuentas involucrados en la transacción. Un valor en una celda indica que la plantilla debita o acredita una cuenta de ese conjunto, en la capa Liquidada o Pendiente. Las celdas vacías significan que la plantilla no involucra ese conjunto de cuentas.

### Facilidad

```
┌─────────────────────────────────────┬──────────────────────┬──────────────────────┐
│ Código de plantilla                 │ Ómnibus de facilidad │ Remanente facilidad  │
├─────────────────────────────────────┼──────────────────────┼──────────────────────┤
│ CREATE_CREDIT_FACILITY_PROPOSAL     │ Débito (Pendiente)   │ Crédito (Pendiente)  │
├─────────────────────────────────────┼──────────────────────┼──────────────────────┤
│ ACTIVATE_CREDIT_FACILITY            │ Crédito (Pendiente)  │ Débito (Pendiente)   │
│                                     │ Débito (Liquidado)   │ Crédito (Liquidado)  │
└─────────────────────────────────────┴──────────────────────┴──────────────────────┘
```

### Desembolsos

| Código de plantilla                     | Remanente facilidad                            | Pendiente sin cobertura                         | Cartera desembolsada                            | Ómnibus de depósitos (externo)                  |
| --------------------------------------- | ----------------------------------------------- | ----------------------------------------------- | ----------------------------------------------- | ----------------------------------------------- |
| INITIAL_DISBURSAL                       | Débito (Liquidado)                              | Crédito (Liquidado)                             | Débito (Liquidado)                              | Crédito (Liquidado)                             |
| INITIATE_CREDIT_FACILITY_DISBURSAL      | Débito (Liquidado)<br/>Crédito (Pendiente)       | Crédito (Liquidado)<br/>Débito (Pendiente)       | –                                               | –                                               |
| CONFIRM_DISBURSAL                       | Débito (Pendiente)                              | Crédito (Pendiente)                             | Débito (Liquidado)                              | Crédito (Liquidado)                             |
| CANCEL_DISBURSAL                        | Débito (Pendiente)<br/>Crédito (Liquidado)       | Crédito (Pendiente)<br/>Débito (Liquidado)       | –                                               | –                                               |

### Interés

```
┌────────────────────────────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│ Código de plantilla                │ Interés por │ Ingreso por │ Int. añadido│ Pendiente   │
│                                    │ cobrar      │ interés     │ a oblig.    │ sin cobertura│
│                                    │             │             │ ómnibus     │             │
├────────────────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│ CREDIT_FACILITY_ACCRUE_INTEREST    │ Débito (Pendiente)│ Crédito (Pendiente)│             │             │
├────────────────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│ CREDIT_FACILITY_POST_ACCRUED_      │ Crédito (Pendiente)│ Débito (Pendiente)│             │             │
│ INTEREST                           │ Débito (Liquidado)│ Crédito (Liquidado)│ Débito (Liquidado)│ Crédito (Liquidado)│
└────────────────────────────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
```

### Comisiones

```
┌─────────────────────┬────────────────────┬──────────────┐
│ Código de plantilla │ Cartera desembolsada│ Ingreso por  │
│                     │                    │ comisiones   │
├─────────────────────┼────────────────────┼──────────────┤
│ ADD_STRUCTURING_FEE │ Débito (Liquidado) │ Crédito (Liquidado) │
└─────────────────────┴────────────────────┴──────────────┘
```
