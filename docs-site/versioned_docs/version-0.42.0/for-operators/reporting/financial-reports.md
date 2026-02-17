---
id: financial-reports
title: Financial Reports
sidebar_position: 2
---

# Financial Reports

This document describes available financial reports, their structure, and how to generate them.

## Trial Balance

Shows balances of all accounting accounts for a given period.

### Structure

| Column | Description |
|--------|-------------|
| Account | Account code and name |
| Debit | Total debit movements |
| Credit | Total credit movements |
| Balance | Resulting balance |

### Generation

```graphql
query GetTrialBalance($input: TrialBalanceInput!) {
  trialBalance(input: $input) {
    accounts {
      code
      name
      debit
      credit
      balance
    }
    totals {
      debit
      credit
    }
    asOfDate
  }
}
```

## Balance Sheet

Presents the institution's financial position at a given date.

### Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                    BALANCE SHEET                                │
│                                                                  │
│  ASSETS                          │  LIABILITIES AND EQUITY      │
│  ─────────────────────────────────┼───────────────────────────  │
│  Current Assets                  │  Current Liabilities         │
│    Cash                          │    Deposits                  │
│    Short-term Loans              │    Obligations               │
│                                  │                              │
│  Non-Current Assets              │  Non-Current Liabilities     │
│    Long-term Loans               │    Long-term Debt            │
│    Fixed Assets                  │                              │
│                                  │  Equity                      │
│                                  │    Capital                   │
│                                  │    Retained Earnings         │
└─────────────────────────────────────────────────────────────────┘
```

## Income Statement

Shows revenue, expenses, and profit for a period.

### Sections

| Section | Components |
|---------|------------|
| Revenue | Loan interest, fees |
| Financial Expenses | Interest paid |
| Operating Expenses | Salaries, administration |
| Net Income | Revenue - Expenses |

## Portfolio Reports

### Credit Portfolio

```graphql
query GetCreditPortfolioReport($asOfDate: Date!) {
  creditPortfolioReport(asOfDate: $asOfDate) {
    facilities {
      id
      customer {
        name
      }
      principal
      outstanding
      status
      interestRate
      maturityDate
    }
    summary {
      totalFacilities
      totalPrincipal
      totalOutstanding
    }
  }
}
```

### Delinquency Report

Analysis of portfolio by days past due.

| Category | Description |
|----------|-------------|
| Current | No delay |
| 1-30 days | Minor delay |
| 31-60 days | Moderate delay |
| 61-90 days | Significant delay |
| > 90 days | Past due portfolio |

## Report Scheduling

### Configure Automatic Report

```graphql
mutation ScheduleReport($input: ReportScheduleInput!) {
  reportSchedule(input: $input) {
    schedule {
      id
      reportType
      frequency
      nextRun
      recipients
    }
  }
}
```

### Available Frequencies

| Frequency | Description |
|-----------|-------------|
| DAILY | Every day |
| WEEKLY | Weekly |
| MONTHLY | Monthly |
| QUARTERLY | Quarterly |
| YEARLY | Annual |

## Permissions Required

| Operation | Permission |
|-----------|---------|
| View financial reports | REPORT_FINANCIAL_READ |
| View portfolio reports | REPORT_PORTFOLIO_READ |
| View regulatory reports | REPORT_REGULATORY_READ |
| Export reports | REPORT_EXPORT |
| Schedule reports | REPORT_SCHEDULE |

