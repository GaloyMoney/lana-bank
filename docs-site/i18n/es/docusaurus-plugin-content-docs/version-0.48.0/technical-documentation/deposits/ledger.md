---
id: ledger
title: Libro Mayor
sidebar_position: 3
---

# Resumen del Libro Mayor

Este documento describe los conjuntos de cuentas creados por el módulo durante la inicialización, su contexto contable y las plantillas de transacciones que estructuran el flujo de fondos entre los conjuntos de cuentas.

## Conjuntos de Cuentas Omnibus

Utilizados para representar entradas/salidas hacia/desde cuentas de depósito en el libro mayor. 

```
Ref: deposit-omnibus-account-set
Name: Conjunto de Cuentas Omnibus de Depósito
Category: Activo
Normal Balance: Débito
Account Creation: Compartida (1 cuenta: deposit-omnibus-account)
```

## Conjuntos de Cuentas Resumen

Utilizados para agrupar las cuentas de clientes creadas por tipo de cliente.

```
Ref: deposit-individual-account-set
Name: Conjunto de Cuentas de Depósito Individual
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: deposit-government-entity-account-set
Name: Conjunto de Cuentas de Depósito de Entidad Gubernamental
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: deposit-private-company-account-set
Name: Conjunto de Cuentas de Depósito de Empresa Privada
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: deposit-bank-account-set
Name: Conjunto de Cuentas de Depósito Bancario
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: deposit-financial-institution-account-set
Name: Conjunto de Cuentas de Depósito de Institución Financiera
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: deposit-non-domiciled-company-account-set
Name: Conjunto de Cuentas de Depósito de Empresa No Domiciliada
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
```

## Conjuntos de Cuentas Resumen Congeladas

Utilizados para agrupar las cuentas de depósito de clientes congeladas por tipo de cliente.

```
Ref: frozen-deposit-individual-account-set
Name: Conjunto de Cuentas de Depósito Individual Congeladas
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: frozen-deposit-government-entity-account-set
Name: Conjunto de Cuentas de Depósito de Entidad Gubernamental Congeladas
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: frozen-deposit-private-company-account-set
Name: Conjunto de Cuentas de Depósito de Empresa Privada Congeladas
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: frozen-deposit-bank-account-set
Name: Conjunto de Cuentas de Depósito Bancario Congeladas
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: frozen-deposit-financial-institution-account-set
Name: Conjunto de Cuentas de Depósito de Institución Financiera Congeladas
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
────────────────────────────────────────
Ref: frozen-deposit-non-domiciled-company-account-set
Name: Conjunto de Cuentas de Depósito de Empresa No Domiciliada Congeladas
Category: Pasivo
Normal Balance: Crédito
Account Creation: Por cliente
```

## Plantillas de Transacciones de Depósito y Retiro

Flujos definidos de fondos entre cuentas de clientes (creadas bajo cuentas resumen) y la cuenta ómnibus.

```
┌───────────────────┬─────────────────────────────────────────┬──────────────────────────────────────────┬──────────────────────────────────────────┐
│  Template Code    │ Operation                               │ Omnibus                                  │ Customer Account Set                     │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ RECORD_DEPOSIT    │ Registrar depósito entrante             │ Debitada                                 │ Acreditada                               │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ INITIATE_WITHDRAW │ Iniciar retiro (liquidado a pendiente)  │ Acreditada (Liquidado), Debitada (Pendiente) │ Debitada (Liquidado), Acreditada (Pendiente) │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ CONFIRM_WITHDRAW  │ Completar retiro                        │ Acreditada (Pendiente)                   │ Debitada (Pendiente)                     │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ DENY_WITHDRAW     │ Rechazar retiro (pendiente a liquidado) │ Acreditada (Pendiente), Debitada (Liquidado) │ Debitada (Pendiente), Acreditada (Liquidado) │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ CANCEL_WITHDRAW   │ Cancelar retiro (pendiente a liquidado) │ Acreditada (Pendiente), Debitada (Liquidado) │ Debitada (Pendiente), Acreditada (Liquidado) │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ REVERT_DEPOSIT    │ Revertir un depósito registrado         │ Acreditada                               │ Debitada                                 │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ REVERT_WITHDRAW   │ Revertir un retiro completado           │ Debitada                                 │ Acreditada                               │
└───────────────────┴─────────────────────────────────────────┴──────────────────────────────────────────┴──────────────────────────────────────────┘
```

## Plantillas de Transacciones de Congelación y Descongelación

Flujos definidos de fondos para congelar y descongelar saldos depositados en cuentas de clientes.

```
┌──────────────────┬───────────────────────┬────────────────────────┬────────────────────────┐
│  Template Code   │       Operation       │ Active Deposit Account │ Frozen Deposit Account │
├──────────────────┼───────────────────────┼────────────────────────┼────────────────────────┤
│ FREEZE_ACCOUNT   │ Bloquear fondos del cliente │ Debitada               │ Acreditada             │
├──────────────────┼───────────────────────┼────────────────────────┼────────────────────────┤
│ UNFREEZE_ACCOUNT │ Desbloquear fondos del cliente │ Acreditada             │ Debitada               │
└──────────────────┴───────────────────────┴────────────────────────┴────────────────────────┘
```
