---
id: closing
title: Closing
sidebar_position: 2
---

# Closing - Transfer Net Income from ProfitAndLossStatement to BalanceSheet

## Offsetting Effective Balances of ProfitAndLossStatement Accounts

### Negative Normal Balance Types

Cala effective balances can be negative. Offsetting negative normal balances with a closing transaction entry works different than a positive normal balance type.

```rust
pub fn settled(&self) -> Decimal {
    if self.direction == DebitOrCredit::Credit {
        self.details.settled.cr_balance - self.details.settled.dr_balance
    } else {
        self.details.settled.dr_balance - self.details.settled.cr_balance
    }
}
```

### Contra-Accounts

A contra-account is an `Account` of an `AccountSet` on the `ChartOfAccounts`, that has a different normal balance-type than its parent.

Example, `lana-bank` operator accounts with a loan-loss provision. On a given month, the realized loan-losses were less than provision for a period. An Accountant/CFO, will make a manual transaction with an entry crediting a credit-normal `Expense` account - reducing the realized losses.
