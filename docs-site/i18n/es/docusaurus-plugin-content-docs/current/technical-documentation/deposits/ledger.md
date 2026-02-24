---
id: ledger
title: Libro Mayor
sidebar_position: 3
---

# Resumen del Libro Mayor

Este documento describe los conjuntos de cuentas creados por el módulo durante la inicialización, su contexto contable y las plantillas de transacción que estructuran el flujo de fondos entre conjuntos de cuentas.

## Conjuntos de Cuentas Ómnibus

Se usan para representar entradas y salidas hacia/desde cuentas de depósito en el libro mayor.

```
Referencia: deposit-omnibus-account-set
Nombre: Conjunto de Cuentas Ómnibus de Depósitos
Categoría: Activo
Balance normal: Débito
Creación de cuenta: Compartida (1 cuenta: deposit-omnibus-account)
```

## Conjuntos de Cuentas Resumen

Se usan para agrupar cuentas de clientes creadas por tipo de cliente.

```
Referencia: deposit-individual-account-set
Nombre: Conjunto de Cuentas de Depósito Individual
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: deposit-government-entity-account-set
Nombre: Conjunto de Cuentas de Depósito para Entidad Gubernamental
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: deposit-private-company-account-set
Nombre: Conjunto de Cuentas de Depósito para Empresa Privada
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: deposit-bank-account-set
Nombre: Conjunto de Cuentas de Depósito para Banco
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: deposit-financial-institution-account-set
Nombre: Conjunto de Cuentas de Depósito para Institución Financiera
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: deposit-non-domiciled-company-account-set
Nombre: Conjunto de Cuentas de Depósito para Empresa No Domiciliada
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
```

## Conjuntos de Cuentas Resumen Congeladas

Se usan para agrupar cuentas de depósito congeladas por tipo de cliente.

```
Referencia: frozen-deposit-individual-account-set
Nombre: Conjunto de Cuentas de Depósito Congelado Individual
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: frozen-deposit-government-entity-account-set
Nombre: Conjunto de Cuentas de Depósito Congelado para Entidad Gubernamental
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: frozen-deposit-private-company-account-set
Nombre: Conjunto de Cuentas de Depósito Congelado para Empresa Privada
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: frozen-deposit-bank-account-set
Nombre: Conjunto de Cuentas de Depósito Congelado para Banco
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: frozen-deposit-financial-institution-account-set
Nombre: Conjunto de Cuentas de Depósito Congelado para Institución Financiera
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
────────────────────────────────────────
Referencia: frozen-deposit-non-domiciled-company-account-set
Nombre: Conjunto de Cuentas de Depósito Congelado para Empresa No Domiciliada
Categoría: Pasivo
Balance normal: Crédito
Creación de cuenta: Por cliente
```

## Plantillas de Transacción para Depósitos y Retiros

Definen los flujos de fondos entre las cuentas del cliente (creadas bajo cuentas resumen) y la cuenta ómnibus.

```
┌───────────────────┬─────────────────────────────────────────┬──────────────────────────────────────────┬──────────────────────────────────────────┐
│ Código de plantilla │ Operación                            │ Ómnibus                                  │ Conjunto de cuentas del cliente          │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ RECORD_DEPOSIT    │ Registrar depósito entrante             │ Débito                                   │ Crédito                                  │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ INITIATE_WITHDRAW │ Iniciar retiro (de liquidado a pendiente)│ Crédito (Liquidado), Débito (Pendiente) │ Débito (Liquidado), Crédito (Pendiente)  │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ CONFIRM_WITHDRAW  │ Confirmar retiro                        │ Crédito (Pendiente)                      │ Débito (Pendiente)                       │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ DENY_WITHDRAW     │ Denegar retiro (de pendiente a liquidado)│ Crédito (Pendiente), Débito (Liquidado) │ Débito (Pendiente), Crédito (Liquidado)  │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ CANCEL_WITHDRAW   │ Cancelar retiro (de pendiente a liquidado)│ Crédito (Pendiente), Débito (Liquidado)│ Débito (Pendiente), Crédito (Liquidado)  │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ REVERT_DEPOSIT    │ Revertir un depósito registrado         │ Crédito                                  │ Débito                                   │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ REVERT_WITHDRAW   │ Revertir un retiro confirmado           │ Débito                                   │ Crédito                                  │
└───────────────────┴─────────────────────────────────────────┴──────────────────────────────────────────┴──────────────────────────────────────────┘
```

## Plantillas de Transacción para Congelar y Descongelar

Definen los flujos de fondos para congelar y descongelar saldos de cuentas de depósito.

```
┌──────────────────┬───────────────────────┬────────────────────────┬────────────────────────┐
│ Código de plantilla │ Operación          │ Cuenta de depósito activa │ Cuenta de depósito congelada │
├──────────────────┼───────────────────────┼────────────────────────┼────────────────────────┤
│ FREEZE_ACCOUNT   │ Bloquear fondos del cliente │ Débito          │ Crédito                │
├──────────────────┼───────────────────────┼────────────────────────┼────────────────────────┤
│ UNFREEZE_ACCOUNT │ Desbloquear fondos del cliente │ Crédito      │ Débito                 │
└──────────────────┴───────────────────────┴────────────────────────┴────────────────────────┘
```
