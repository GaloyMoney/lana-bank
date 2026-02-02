---
id: cala-ledger-integration
title: Cala Ledger Integration
sidebar_position: 8
---

# Integration with Cala Ledger

This document describes the integration with Cala Ledger for double-entry accounting.

![Accounting and Ledger Integration](/img/architecture/accounting-1.png)

## Overview

Cala Ledger provides:

- Double-entry bookkeeping
- Account hierarchy management
- Balance calculation
- Transaction templates

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    LEDGER INTEGRATION                           │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Lana Domain Services                  │   │
│  │    (Credit, Deposit, Accounting)                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    LedgerAdapter                         │   │
│  │              (Domain-specific operations)                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      Cala Ledger                         │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐        │   │
│  │  │  Accounts  │  │ Transactions│  │  Balances  │        │   │
│  │  └────────────┘  └────────────┘  └────────────┘        │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Account Hierarchy

```
                    ┌──────────────┐
                    │     Root     │
                    │   Account    │
                    └──────────────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
    ┌──────────┐     ┌──────────┐     ┌──────────┐
    │  Assets  │     │Liabilities│    │  Equity  │
    └──────────┘     └──────────┘     └──────────┘
          │                │
    ┌─────┴─────┐    ┌─────┴─────┐
    ▼           ▼    ▼           ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│  Cash  │ │ Loans  │ │Deposits│ │ Debt   │
└────────┘ └────────┘ └────────┘ └────────┘
```

## Account Types

| Account | Type | Purpose |
|---------|------|---------|
| Cash | Asset | Bank's cash holdings |
| Loans Receivable | Asset | Outstanding loan principal |
| Customer Deposits | Liability | Customer deposit balances |
| Interest Income | Revenue | Earned interest |
| Interest Expense | Expense | Paid interest |

## Transaction Templates

### Deposit Recording

```
DEBIT:  Cash (Asset)              $1,000
CREDIT: Customer Deposit (Liability)  $1,000
```

### Loan Disbursement

```
DEBIT:  Loans Receivable (Asset)  $10,000
CREDIT: Cash (Asset)              $10,000
```

### Interest Accrual

```
DEBIT:  Interest Receivable (Asset)  $100
CREDIT: Interest Income (Revenue)    $100
```

### Loan Payment

```
DEBIT:  Cash (Asset)                 $500
CREDIT: Loans Receivable (Asset)     $400
CREDIT: Interest Receivable (Asset)  $100
```

## Ledger Adapter

```rust
pub struct DepositLedger {
    cala: CalaClient,
}

impl DepositLedger {
    pub async fn record_deposit(
        &self,
        account_id: AccountId,
        amount: UsdCents,
    ) -> Result<TransactionId> {
        let entries = vec![
            Entry::debit(self.cash_account, amount),
            Entry::credit(account_id, amount),
        ];

        self.cala.post_transaction(entries).await
    }

    pub async fn process_withdrawal(
        &self,
        account_id: AccountId,
        amount: UsdCents,
    ) -> Result<TransactionId> {
        let entries = vec![
            Entry::debit(account_id, amount),
            Entry::credit(self.cash_account, amount),
        ];

        self.cala.post_transaction(entries).await
    }
}
```

## Balance Queries

### Account Balance

```rust
pub async fn get_balance(&self, account_id: AccountId) -> Result<Balance> {
    self.cala.get_balance(account_id).await
}

pub struct Balance {
    pub settled: UsdCents,
    pub pending: UsdCents,
    pub available: UsdCents,
}
```

### Trial Balance

```rust
pub async fn trial_balance(&self, as_of: DateTime<Utc>) -> Result<TrialBalance> {
    self.cala.trial_balance(as_of).await
}
```

## Transaction Journaling

All transactions are recorded with:

- Unique transaction ID
- Timestamp
- Correlation ID (for tracing)
- Description
- Entry details

## Consistency Guarantees

- Atomic transactions (all-or-nothing)
- Balanced entries (debits = credits)
- Immutable transaction history
- Audit trail for all changes

