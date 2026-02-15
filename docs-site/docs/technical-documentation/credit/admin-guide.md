---
id: admin-guide
title: "Admin Panel Guide: Credit Facilities"
sidebar_position: 8
---

# Credit Facility Walkthrough

This guide walks through the complete credit facility lifecycle using the admin panel — from
creating a proposal through disbursal of funds. Each step includes a screenshot showing
exactly what you will see in the interface.

A credit facility is a legally binding lending agreement between the bank and a customer. It
establishes a maximum credit limit, specifies loan terms (interest rates, fees, risk parameters),
and defines the collateral requirements that must be met before funds can be disbursed.

---

## 1. Creating a Credit Facility Proposal

A credit facility begins as a **proposal** created by a bank operator on behalf of a customer.
The proposal specifies the facility amount and links it to a **Terms Template** that defines
interest rates, collateral requirements, and other parameters.

**Step 1.** From a customer's page, click the **Create** button. A dropdown menu appears
with available actions.

![Click Create button](/img/screenshots/current/en/credit-facilities.cy.ts/01_click_create_proposal_button.png)

**Step 2.** Select **Credit Facility** to open the proposal creation form.

![Open proposal form](/img/screenshots/current/en/credit-facilities.cy.ts/02_open_proposal_form.png)

**Step 3.** Enter the desired facility amount and select a Terms Template from the dropdown.
The terms template defines the interest rate, CVL thresholds, duration, and fee structure.

![Enter facility amount](/img/screenshots/current/en/credit-facilities.cy.ts/03_enter_facility_amount.png)

**Step 4.** Click **Create** to submit the proposal.

![Submit proposal form](/img/screenshots/current/en/credit-facilities.cy.ts/04_submit_proposal_form.png)

**Step 5.** The proposal is created successfully. You are redirected to the proposal detail
page where you can review all the details. The initial status is **Pending Customer Approval**.

![Proposal created successfully](/img/screenshots/current/en/credit-facilities.cy.ts/05_proposal_created_success.png)

**Step 6.** Verify the proposal appears in the credit facility proposals list.

![Proposal in list](/img/screenshots/current/en/credit-facilities.cy.ts/06_proposal_in_list.png)

---

## 2. Customer Acceptance and Internal Approval

Before a facility can proceed, the customer must accept the proposal terms. After customer
acceptance, the proposal goes through an internal approval process managed by the governance
module, where committee members review and vote on the proposal.

### Customer Acceptance

**Step 7.** Navigate to the proposal detail page.

![Visit proposal page](/img/screenshots/current/en/credit-facilities.cy.ts/07_visit_proposal_page.png)

**Step 8.** Click the **Customer Accepts** button to record the customer's acceptance of
the proposal terms.

![Customer approval button](/img/screenshots/current/en/credit-facilities.cy.ts/08_customer_approval_button.png)

**Step 9.** Confirm the customer acceptance in the dialog.

![Customer approval dialog](/img/screenshots/current/en/credit-facilities.cy.ts/09_customer_approval_dialog.png)

**Step 10.** The proposal status changes to **Pending Approval**, indicating it now requires
internal committee approval.

![Proposal pending approval](/img/screenshots/current/en/credit-facilities.cy.ts/10_proposal_pending_approval_status.png)

### Internal Approval

**Step 11.** Click the **Approve** button to begin the internal approval process.

![Approve proposal button](/img/screenshots/current/en/credit-facilities.cy.ts/11_approve_proposal_button.png)

**Step 12.** Confirm the approval in the dialog. Depending on the governance policy, multiple
committee members may need to approve before the proposal is fully approved.

![Approve proposal dialog](/img/screenshots/current/en/credit-facilities.cy.ts/12_approve_proposal_dialog.png)

**Step 13.** The proposal status changes to **Approved**.

![Proposal approved](/img/screenshots/current/en/credit-facilities.cy.ts/13_proposal_approved_status.png)

**Step 14.** Click **View Pending Facility** to navigate to the newly created pending credit
facility.

![View pending facility button](/img/screenshots/current/en/credit-facilities.cy.ts/14_view_pending_facility_button.png)

---

## 3. Collateralization and Activation

After approval, the proposal becomes a **Pending Credit Facility**. The customer must deposit
Bitcoin collateral that meets the Collateral Value to Loan (CVL) ratio defined in the facility's
terms. The facility activates automatically once the collateral threshold is met.

**Step 15.** The pending facility page shows the current status as **Pending Collateralization**
and displays the collateral requirements.

![Pending facility initial state](/img/screenshots/current/en/credit-facilities.cy.ts/15_pending_facility_initial_state.png)

**Step 16.** Click the **Update Collateral** button to record a collateral deposit.

![Click update collateral](/img/screenshots/current/en/credit-facilities.cy.ts/16_click_update_collateral_button.png)

**Step 17.** Enter the new collateral amount. The page shows the target amount needed to meet
the initial CVL requirement.

![Enter collateral value](/img/screenshots/current/en/credit-facilities.cy.ts/17_enter_new_collateral_value.png)

**Step 18.** The collateral is updated successfully.

![Collateral updated](/img/screenshots/current/en/credit-facilities.cy.ts/18_collateral_updated.png)

**Step 19.** The pending facility status changes to **Completed**, indicating the collateral
requirements have been met and the facility has been activated.

![Pending facility completed](/img/screenshots/current/en/credit-facilities.cy.ts/19_pending_facility_completed.png)

**Step 20.** Click **View Facility** to navigate to the now-active credit facility.

![View facility button](/img/screenshots/current/en/credit-facilities.cy.ts/20_view_facility_button.png)

**Step 21.** Verify the credit facility status is **Active**. At this point, interest accrual
begins and the customer can request disbursals.

![Verify active status](/img/screenshots/current/en/credit-facilities.cy.ts/21_verify_active_status.png)

**Step 22.** The facility also appears in the credit facilities list.

![Credit facility in list](/img/screenshots/current/en/credit-facilities.cy.ts/22_credit_facility_in_list.png)

---

## 4. Disbursal

With an active credit facility, the customer can receive **disbursals** — principal amounts
sent to the customer from the facility. Each disbursal goes through its own approval process
and creates obligations that track the repayment schedule.

**Step 23.** From the active credit facility page, click **Create** and then **Disbursal** to
initiate a new disbursal.

![Initiate disbursal](/img/screenshots/current/en/credit-facilities.cy.ts/23_click_initiate_disbursal_button.png)

**Step 24.** Enter the disbursal amount. The amount must be within the facility's available
credit limit.

![Enter disbursal amount](/img/screenshots/current/en/credit-facilities.cy.ts/24_enter_disbursal_amount.png)

**Step 25.** Submit the disbursal request.

![Submit disbursal](/img/screenshots/current/en/credit-facilities.cy.ts/25_submit_disbursal_request.png)

**Step 26.** The disbursal is created and you are redirected to the disbursal detail page.

![Disbursal page](/img/screenshots/current/en/credit-facilities.cy.ts/26_disbursal_page.png)

**Step 27.** Click **Approve** to approve the disbursal. Like proposals, disbursals may
require approval from multiple committee members depending on the governance policy.

![Approve disbursal](/img/screenshots/current/en/credit-facilities.cy.ts/27_approve.png)

**Step 28.** The disbursal status changes to **Confirmed**. The funds have been credited to
the customer's deposit account, and a corresponding obligation has been created to track
repayment.

![Disbursal confirmed](/img/screenshots/current/en/credit-facilities.cy.ts/28_verify_disbursal_status_confirmed.png)

**Step 29.** The disbursal appears in the disbursals list.

![Disbursal in list](/img/screenshots/current/en/credit-facilities.cy.ts/29_disbursal_in_list.png)
