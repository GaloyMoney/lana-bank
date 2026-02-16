---
id: disbursal
title: Disbursal
sidebar_position: 3
---

# Disbursal

Disbursals are the **principal amounts** sent to the customer.
Each disbursal records the amount released, links it to the facility and emits events that create new obligations.
Those obligations track the principal (and any fees) that must be repaid according to the facility terms.

## Preconditions and Validation

Before a disbursal can be initiated, the domain applies strict checks:

- Facility must be `Active`.
- Disbursal date must be before facility maturity.
- Customer verification requirements must be satisfied (when enabled by policy).
- Disbursal policy must allow a new disbursal (`SingleDisbursal` vs `MultipleDisbursal` behavior).
- Post-disbursal CVL must remain at or above `margin_call_cvl`.

These controls prevent under-collateralized or out-of-policy lending events from being created.

## Status and Outcome Model

Operators typically see these status transitions:

- `New`: disbursal initialized and awaiting governance decision.
- `Approved`: governance approval threshold reached.
- `Confirmed`: disbursal settled; funds credited and obligation created.
- `Denied`: governance rejected; disbursal cancelled/reversed.

In practical terms, only `Confirmed` means funds are fully released and repayment tracking is
active in obligations.

## Relationship to Obligations and Interest

A confirmed disbursal creates a principal obligation. That obligation then participates in the
interest lifecycle:

- periodic accrual processing posts interest amounts,
- interest can create additional interest-type obligations,
- borrower payments allocate against outstanding obligations using allocation rules.

For operators, this means disbursal confirmation is the starting point of long-lived repayment and
risk monitoring, not the end of the workflow.

## Admin Panel Walkthrough: Create and Approve a Disbursal

This flow continues from an already active credit facility and shows how operators create and approve a disbursal.

**Step 23.** From the active facility page, click **Create** and then **Disbursal**.

![Initiate disbursal](/img/screenshots/current/en/credit-facilities.cy.ts/23_click_initiate_disbursal_button.png)

**Step 24.** Enter the disbursal amount.

![Enter disbursal amount](/img/screenshots/current/en/credit-facilities.cy.ts/24_enter_disbursal_amount.png)

**Step 25.** Submit the disbursal request.

![Submit disbursal](/img/screenshots/current/en/credit-facilities.cy.ts/25_submit_disbursal_request.png)

**Step 26.** Confirm you are redirected to the disbursal detail page.

![Disbursal page](/img/screenshots/current/en/credit-facilities.cy.ts/26_disbursal_page.png)

**Step 27.** Click **Approve** to run governance approval.

![Approve disbursal](/img/screenshots/current/en/credit-facilities.cy.ts/27_approve.png)

**Step 28.** Verify the status changes to **Confirmed**.

![Disbursal confirmed](/img/screenshots/current/en/credit-facilities.cy.ts/28_verify_disbursal_status_confirmed.png)

**Step 29.** Confirm the disbursal appears in the disbursals list.

![Disbursal in list](/img/screenshots/current/en/credit-facilities.cy.ts/29_disbursal_in_list.png)

## What To Verify After Step 29

- Disbursal status is `Confirmed`.
- The disbursal is visible under the expected facility and customer.
- Facility history reflects execution/settlement activity.
- Repayment views show obligation impact for the new principal.
