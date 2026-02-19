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

Trial balance reports can be generated from the **Reports** section of the admin panel by selecting the desired period.

## Balance Sheet

Presents the institution's financial position at a given date.

### Structure

```mermaid
graph LR
    subgraph Assets
        CA["**Current Assets**<br/>Cash<br/>Short-term Loans"]
        NCA["**Non-Current Assets**<br/>Long-term Loans<br/>Fixed Assets"]
    end
    subgraph LiabilitiesAndEquity["Liabilities & Equity"]
        CL["**Current Liabilities**<br/>Deposits<br/>Obligations"]
        NCL["**Non-Current Liabilities**<br/>Long-term Debt"]
        EQ["**Equity**<br/>Capital<br/>Retained Earnings"]
    end
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

Credit portfolio reports are available from the **Reports** section and show all active facilities with their current status.

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

Report schedules can be configured from the **Reports** > **Scheduling** section in the admin panel.

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

