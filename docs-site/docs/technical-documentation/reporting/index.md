---
id: index
title: Reporting System
sidebar_position: 1
---

# Financial Reporting System

The reporting system provides financial reports for operational management and regulatory compliance.

## Purpose

The reporting system enables:
- Financial statement generation
- Regulatory reports
- Portfolio analysis
- Audit reports

## Report Types

### Financial Reports

| Report | Description | Frequency |
|--------|-------------|-----------|
| Trial Balance | All account balances | Daily/Monthly |
| Balance Sheet | Financial position statement | Monthly |
| Income Statement | Revenue and expenses | Monthly |

### Operational Reports

| Report | Description | Frequency |
|--------|-------------|-----------|
| Credit Portfolio | Credit facility status | Daily |
| Deposits | Deposit position | Daily |
| Collateral | Collateral valuation | Daily |

### Regulatory Reports

| Report | Description | Frequency |
|--------|-------------|-----------|
| Credit Concentration | Exposure by customer | Monthly |
| Delinquency | Past due portfolio | Monthly |
| Capital | Capital ratios | Quarterly |

## Report Access

### Admin Panel

1. Navigate to **Reports**
2. Select report type
3. Configure parameters:
   - Period
   - Filters
   - Output format
4. Generate report

### Export Formats

| Format | Usage |
|--------|-------|
| PDF | Formal presentation |
| Excel | Additional analysis |
| CSV | System integration |

## Related Documentation

- [Financial Reports](financial-reports) - Financial report details

## Admin Panel Walkthrough: Regulatory Reports

Regulatory reports are generated asynchronously. After triggering a run, operators should monitor
state transitions (`queued` -> `running` -> `success`/`failed`) and only generate download links
after successful completion.

**Step 1.** Open regulatory reporting and click **Generate Report**.

![Generate report button](/img/screenshots/current/en/reporting.cy.ts/1_generate_report_button.png)

Verification checklist:
- report run appears in the list,
- status updates are reflected in UI,
- download links are generated only when run state is successful.

