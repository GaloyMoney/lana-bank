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
