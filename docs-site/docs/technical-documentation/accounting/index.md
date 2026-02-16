---
id: index
title: Accounting
sidebar_position: 1
---

# Accounting

## ChartOfAccounts

### ChartOfAccountsIntegrationConfig

## FiscalYear

### Financial Statements

#### ProfitAndLossStatement

#### BalanceSheet

## Operational Model

Accounting pages in the admin panel expose ledger-structured views backed by Cala double-entry
bookkeeping. Operators typically use:
- **Chart of Accounts** to inspect hierarchy and account-level activity.
- **Trial Balance** to validate ledger balance consistency.
- **Balance Sheet** for position (assets, liabilities, equity).
- **Profit and Loss** for period performance.

## Admin Panel Walkthrough: Module Configuration

Module configuration maps operational flows (deposits and credit) to chart-of-accounts parent
codes. These mappings are critical because they determine where transactions are posted.

**Step 1.** Open module configuration.

![Modules configuration](/img/screenshots/current/en/modules.cy.ts/1_modules_configuration.png)

**Step 2.** Configure deposit accounting mappings.

![Deposit configuration](/img/screenshots/current/en/modules.cy.ts/2_deposit_configuration.png)

**Step 3.** Configure credit accounting mappings.

![Credit configuration](/img/screenshots/current/en/modules.cy.ts/3_credit_configuration.png)

## Admin Panel Walkthrough: Chart of Accounts

**Step 1.** Open chart of accounts and verify hierarchy view.

![Chart of accounts view](/img/screenshots/current/en/chart-of-accounts.cy.ts/2_chart_of_account_view.png)

**Step 2.** Open a ledger account detail to inspect postings.

![Ledger account details](/img/screenshots/current/en/chart-of-accounts.cy.ts/3_ledger_account_details.png)
