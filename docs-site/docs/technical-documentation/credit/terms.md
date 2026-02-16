---
id: terms
title: Terms
sidebar_position: 6
---

# Terms

Terms is a **value object** that captures the parameters under which a credit facility operates.
It is copied into a facility when the facility is created and does not change thereafter.

## Fields

The `TermValues` structure contains the following fields:

- `annual_rate` – interest rate charged on outstanding principal.
- `duration` – total length of the facility.
- `interest_due_duration_from_accrual` – time from interest accrual to when that interest becomes due.
- `obligation_overdue_duration_from_due` – optional grace period before a due obligation becomes overdue.
- `obligation_liquidation_duration_from_due` – optional buffer before an overdue obligation is eligible for liquidation.
- `accrual_cycle_interval` – cadence at which new interest obligations are generated.
- `accrual_interval` – frequency used to calculate accrued interest within a cycle.
- `one_time_fee_rate` – percentage fee taken at disbursal.
- `liquidation_cvl` – collateral value limit that triggers liquidation.
- `margin_call_cvl` – collateral value limit that triggers a margin call.
- `initial_cvl` – collateral value limit required at facility creation.

## Terms Templates

`TermsTemplate` is an entity used to persist a reusable set of term values.
Credit facilities are **not** linked to templates; instead, a template's values are
copied into the facility at creation time.

## Operational Importance

From an operations perspective, terms templates are risk controls, not just configuration:
- `annual_rate` and duration shape borrowing cost and obligation horizon.
- `initial_cvl`, `margin_call_cvl`, and `liquidation_cvl` define collateral safety boundaries.
- `one_time_fee_rate` controls disbursal-time fee behavior.
- disbursal policy controls whether lending is single-shot or multi-draw.

Template quality directly impacts approval quality, activation behavior, and downstream collateral
monitoring.

## Admin Panel Walkthrough: Create and Update Terms Template

### A) Create template

**Step 1.** Open terms templates page.

![Visit terms templates page](/img/screenshots/current/en/terms-templates.cy.ts/1_visit_terms_templates_page.png)

**Step 2.** Click **Create**.

![Click create template](/img/screenshots/current/en/terms-templates.cy.ts/2_click_create_button.png)

**Step 3.** Enter unique template name.

![Enter template name](/img/screenshots/current/en/terms-templates.cy.ts/3_enter_template_name.png)

**Step 4.** Enter annual rate.

![Enter annual rate](/img/screenshots/current/en/terms-templates.cy.ts/4_enter_annual_rate.png)

**Step 5.** Enter duration units.

![Enter duration units](/img/screenshots/current/en/terms-templates.cy.ts/5_enter_duration_units.png)

**Step 6.** Enter `initial_cvl`.

![Enter initial CVL](/img/screenshots/current/en/terms-templates.cy.ts/6_enter_initial_cvl.png)

**Step 7.** Enter `margin_call_cvl`.

![Enter margin call CVL](/img/screenshots/current/en/terms-templates.cy.ts/7_enter_margin_call_cvl.png)

**Step 8.** Enter `liquidation_cvl`.

![Enter liquidation CVL](/img/screenshots/current/en/terms-templates.cy.ts/8_enter_liquidation_cvl.png)

**Step 9.** Enter one-time fee rate.

![Enter fee rate](/img/screenshots/current/en/terms-templates.cy.ts/9_enter_fee_rate.png)

**Step 10.** Select disbursal policy.

![Select disbursal policy](/img/screenshots/current/en/terms-templates.cy.ts/10_select_disbursal_policy.png)

**Step 11.** Submit template.

![Submit terms template](/img/screenshots/current/en/terms-templates.cy.ts/11_submit_terms_template.png)

**Step 12.** Verify template detail and URL.

![Verify terms template creation](/img/screenshots/current/en/terms-templates.cy.ts/12_verify_terms_template_creation.png)

**Step 13.** Verify template is listed.

![Template in list](/img/screenshots/current/en/terms-templates.cy.ts/13_terms_template_in_list.png)

### B) Update template

**Step 14.** Open template details.

![Template details](/img/screenshots/current/en/terms-templates.cy.ts/14_terms_template_details.png)

**Step 15.** Click **Update**.

![Click update template](/img/screenshots/current/en/terms-templates.cy.ts/15_click_update_button.png)

**Step 16.** Modify selected field(s) (example: annual rate).

![Update annual rate](/img/screenshots/current/en/terms-templates.cy.ts/16_update_annual_rate.png)

**Step 17.** Submit changes.

![Submit template update](/img/screenshots/current/en/terms-templates.cy.ts/17_submit_update.png)

**Step 18.** Verify update success message.

![Template update success](/img/screenshots/current/en/terms-templates.cy.ts/18_update_success.png)
