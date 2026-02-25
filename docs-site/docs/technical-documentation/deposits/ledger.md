---
id: ledger
title: Ledger
sidebar_position: 3
---

# Ledger Overview

This document describes the account sets created by the module on initialization, their accounting context, and the transaction templates that structure the flow of funds between the account sets.

## Omnibus Account Sets

Used to represent inflows/outflows to/from deposit accounts in the ledger. 

```
Ref: deposit-omnibus-account-set
Name: Deposit Omnibus Account Set
Category: Asset
Normal Balance: Debit
Account Creation: Shared (1 account: deposit-omnibus-account)
```

## Summary Account Sets

Used to group created customer accounts by customer type.

```
Ref: deposit-individual-account-set
Name: Deposit Individual Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: deposit-government-entity-account-set
Name: Deposit Government Entity Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: deposit-private-company-account-set
Name: Deposit Private Company Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: deposit-bank-account-set
Name: Deposit Bank Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: deposit-financial-institution-account-set
Name: Deposit Financial Institution Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: deposit-non-domiciled-company-account-set
Name: Deposit Non-Domiciled Company Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
```

## Frozen Summary Account Sets

Used to group frozen customer deposit accounts by customer type.

```
Ref: frozen-deposit-individual-account-set
Name: Frozen Deposit Individual Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: frozen-deposit-government-entity-account-set
Name: Frozen Deposit Government Entity Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: frozen-deposit-private-company-account-set
Name: Frozen Deposit Private Company Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: frozen-deposit-bank-account-set
Name: Frozen Deposit Bank Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: frozen-deposit-financial-institution-account-set
Name: Frozen Deposit Financial Institution Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
────────────────────────────────────────
Ref: frozen-deposit-non-domiciled-company-account-set
Name: Frozen Deposit Non-Domiciled Company Account Set
Category: Liability
Normal Balance: Credit
Account Creation: Per-customer
```

## Deposit and Withdrawal Transaction Templates

Defined flows of funds between customer accounts (created under summary accounts) and the omnibus account.

```
┌───────────────────┬─────────────────────────────────────────┬──────────────────────────────────────────┬──────────────────────────────────────────┐
│  Template Code    │ Operation                               │ Omnibus                                  │ Customer Account Set                     │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ RECORD_DEPOSIT    │ Record incoming deposit                 │ Debited                                  │ Credited                                 │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ INITIATE_WITHDRAW │ Begin withdrawal (settled to pending)   │ Credited (Settled), Debited (Pending)    │ Debited (Settled), Credited (Pending)    │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ CONFIRM_WITHDRAW  │ Complete withdrawal                     │ Credited (Pending)                       │ Debited (Pending)                        │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ DENY_WITHDRAW     │ Reject withdrawal (pending to settled)  │ Credited (Pending), Debited (Settled)    │ Debited (Pending), Credited (Settled)    │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ CANCEL_WITHDRAW   │ Cancel withdrawal (pending to settled)  │ Credited (Pending), Debited (Settled)    │ Debited (Pending), Credited (Settled)    │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ REVERT_DEPOSIT    │ Reverse a recorded deposit              │ Credited                                 │ Debited                                  │
├───────────────────┼─────────────────────────────────────────┼──────────────────────────────────────────┼──────────────────────────────────────────┤
│ REVERT_WITHDRAW   │ Reverse a completed withdrawal          │ Debited                                  │ Credited                                 │
└───────────────────┴─────────────────────────────────────────┴──────────────────────────────────────────┴──────────────────────────────────────────┘
```

## Freeze and Unfreeze Transaction Templates

Defined flows of funds to freeze and unfreeze deposited customer account balances.

```
┌──────────────────┬───────────────────────┬────────────────────────┬────────────────────────┐
│  Template Code   │       Operation       │ Active Deposit Account │ Frozen Deposit Account │
├──────────────────┼───────────────────────┼────────────────────────┼────────────────────────┤
│ FREEZE_ACCOUNT   │ Lock customer funds   │ Debited                │ Credited               │
├──────────────────┼───────────────────────┼────────────────────────┼────────────────────────┤
│ UNFREEZE_ACCOUNT │ Unlock customer funds │ Credited               │ Debited                │
└──────────────────┴───────────────────────┴────────────────────────┴────────────────────────┘
```
