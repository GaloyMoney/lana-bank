---
id: ledger
title: Ledger
sidebar_position: 8
---

# Ledger Overview

This document describes the account sets created by the credit module on initialization, their accounting context, and the transaction templates that structure the flow of funds between accounts.

## Omnibus Account Sets

All omnibus account sets are off-balance sheet and contain a single shared account.

### Facility

```
Ref: credit-facility-omnibus-account-set
Name: Credit Facility Omnibus Account Set
Purpose: Tracks total credit line commitments across all facilities
Category: Off-Balance Sheet
Normal Balance: Debit
Account Creation: Shared (1 account: credit-facility-omnibus-account)
```

### Collateral

```
Ref: credit-collateral-omnibus-account-set
Name: Credit Collateral Omnibus Account Set
Purpose: Tracks total collateral deposited across all facilities
Category: Off-Balance Sheet
Normal Balance: Debit
Account Creation: Shared (1 account: credit-collateral-omnibus-account)
────────────────────────────────────────
Ref: credit-facility-liquidation-proceeds-omnibus-account-set
Name: Credit Facility Liquidation Proceeds Omnibus Account Set
Purpose: Tracks total liquidation proceeds received across all facilities
Category: Off-Balance Sheet
Normal Balance: Debit
Account Creation: Shared (1 account: credit-facility-liquidation-proceeds-omnibus-account)
```

### Interest

```
Ref: credit-interest-added-to-obligations-omnibus-account-set
Name: Credit Interest Added to Obligations Omnibus Account Set
Purpose: Tracks total posted interest added to borrower obligations
Category: Off-Balance Sheet
Normal Balance: Debit
Account Creation: Shared (1 account: credit-interest-added-to-obligations-omnibus-account)
```

### Payments

```
Ref: credit-payments-made-omnibus-account-set
Name: Credit Payments Made Omnibus Account Set
Purpose: Tracks total payments received across all facilities
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Shared (1 account: credit-payments-made-omnibus-account)
```

## Summary Account Sets

All summary account sets aggregate balances/transactions from accounts that are created per credit facility.

### Facility

```
Ref: credit-facility-remaining-account-set
Name: Credit Facility Remaining Account Set
Purpose: Tracks undrawn facility balance
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Per-facility
────────────────────────────────────────
Ref: credit-uncovered-outstanding-account-set
Name: Credit Uncovered Outstanding Account Set
Purpose: Tracks outstanding amount not yet covered by an unapplied payment
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Per-facility
```

### Collateral

```
Ref: credit-collateral-account-set
Name: Credit Collateral Account Set
Purpose: Tracks collateral pledged to the facility
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Per-facility
────────────────────────────────────────
Ref: credit-facility-collateral-in-liquidation-account-set
Name: Credit Facility Collateral In-Liquidation Account Set
Purpose: Tracks collateral in active liquidation
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Per-facility
────────────────────────────────────────
Ref: credit-facility-liquidated-collateral-account-set
Name: Credit Facility Liquidated Collateral Account Set
Purpose: Tracks collateral that has been liquidated
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Per-facility
────────────────────────────────────────
Ref: credit-facility-proceeds-from-liquidation-account-set
Name: Credit Facility Proceeds From Liquidation Account Set
Purpose: Tracks proceeds received from collateral liquidation
Category: Off-Balance Sheet
Normal Balance: Credit
Account Creation: Per-facility
```

### Short-Term Disbursed Receivable

All sets in this group are Asset category, Debit normal balance, per-facility.

Ref pattern: `short-term-credit-{type}-disbursed-receivable-account-set`

Purpose: Tracks principal owed on short-term facilities, by customer type.

Where `{type}` is one of: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Long-Term Disbursed Receivable

All sets in this group are Asset category, Debit normal balance, per-facility.

Ref pattern: `long-term-credit-{type}-disbursed-receivable-account-set`

Purpose: Tracks principal owed on long-term facilities, by customer type.

Where `{type}` is one of: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Overdue Disbursed Receivable

All sets in this group are Asset category, Debit normal balance, per-facility.

Ref pattern: `overdue-credit-{type}-disbursed-receivable-account-set`

Purpose: Tracks principal past due, by customer type.

Where `{type}` is one of: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Defaulted Receivable

```
Ref: credit-disbursed-defaulted-account-set
Name: Credit Disbursed Defaulted Account Set
Purpose: Tracks defaulted principal
Category: Asset
Normal Balance: Debit
Account Creation: Per-facility
────────────────────────────────────────
Ref: credit-interest-defaulted-account-set
Name: Credit Interest Defaulted Account Set
Purpose: Tracks defaulted interest
Category: Asset
Normal Balance: Debit
Account Creation: Per-facility
```

### Short-Term Interest Receivable

All sets in this group are Asset category, Debit normal balance, per-facility.

Ref pattern: `short-term-credit-{type}-interest-receivable-account-set`

Purpose: Tracks interest owed on short-term facilities, by customer type.

Where `{type}` is one of: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Long-Term Interest Receivable

All sets in this group are Asset category, Debit normal balance, per-facility.

Ref pattern: `long-term-credit-{type}-interest-receivable-account-set`

Purpose: Tracks interest owed on long-term facilities, by customer type.

Where `{type}` is one of: `individual`, `government-entity`, `private-company`, `bank`, `financial-institution`, `foreign-agency-or-subsidiary`, `non-domiciled-company`.

### Revenue

```
Ref: credit-interest-income-account-set
Name: Credit Interest Income Account Set
Purpose: Recognized interest revenue
Category: Revenue
Normal Balance: Credit
Account Creation: Per-facility
────────────────────────────────────────
Ref: credit-fee-income-account-set
Name: Credit Fee Income Account Set
Purpose: Recognized fee revenue (structuring fees)
Category: Revenue
Normal Balance: Credit
Account Creation: Per-facility
```

### Payment Holding

```
Ref: credit-payment-holding-account-set
Name: Credit Payment Holding Account Set
Purpose: Temporarily holds payments that are waiting to be applied to obligations
Category: Asset
Normal Balance: Credit
Account Creation: Per-facility
```

## Transaction Templates

Columns to the right of Template Code represent account sets involved in the transaction. A cell value indicates that the template debits (DR) or credits (CR) an account from that account set, on the Settled (S) or Pending (P) layer. Empty cells mean the template does not involve that account set.

### Facility

```
┌─────────────────────────────────────┬──────────────────────┬──────────────────────┐
│ Template Code                       │ Facility Omnibus     │ Facility Remaining   │
├─────────────────────────────────────┼──────────────────────┼──────────────────────┤
│ CREATE_CREDIT_FACILITY_PROPOSAL     │ DR (P)               │ CR (P)               │
├─────────────────────────────────────┼──────────────────────┼──────────────────────┤
│ ACTIVATE_CREDIT_FACILITY            │ CR (P)               │ DR (P)               │
│                                     │ DR (S)               │ CR (S)               │
└─────────────────────────────────────┴──────────────────────┴──────────────────────┘
```

### Disbursals

```
┌──────────────────────────┬─────────────┬─────────────┬─────────────┬──────────────────┐
│ Template Code            │ Facility    │ Uncovered   │ Disbursed   │ Deposit Omnibus  │
│                          │ Remaining   │ Outstanding │ Receivable  │ (external)       │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ INITIAL_DISBURSAL        │ DR (S)      │ CR (S)      │ DR (S)      │ CR (S)           │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ INITIATE_CREDIT_FACILITY │ DR (S)      │ CR (S)      │             │                  │
│ _DISBURSAL               │ CR (P)      │ DR (P)      │             │                  │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ CONFIRM_DISBURSAL        │ DR (P)      │ CR (P)      │ DR (S)      │ CR (S)           │
├──────────────────────────┼─────────────┼─────────────┼─────────────┼──────────────────┤
│ CANCEL_DISBURSAL         │ DR (P)      │ CR (P)      │             │                  │
│                          │ CR (S)      │ DR (S)      │             │                  │
└──────────────────────────┴─────────────┴─────────────┴─────────────┴──────────────────┘
```

### Interest

```
┌────────────────────────────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│ Template Code                      │ Interest    │ Interest    │ Int. Added  │ Uncovered   │
│                                    │ Receivable  │ Income      │ to Oblig.   │ Outstanding │
│                                    │             │             │ Omnibus     │             │
├────────────────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│ CREDIT_FACILITY_ACCRUE_INTEREST    │ DR (P)      │ CR (P)      │             │             │
├────────────────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│ CREDIT_FACILITY_POST_ACCRUED_      │ CR (P)      │ DR (P)      │             │             │
│ INTEREST                           │ DR (S)      │ CR (S)      │ DR (S)      │ CR (S)      │
└────────────────────────────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
```

### Fees

```
┌─────────────────────┬────────────────────┬──────────────┐
│ Template Code       │ Disbursed          │ Fee Income   │
│                     │ Receivable         │              │
├─────────────────────┼────────────────────┼──────────────┤
│ ADD_STRUCTURING_FEE │ DR (S)             │ CR (S)       │
└─────────────────────┴────────────────────┴──────────────┘
```
