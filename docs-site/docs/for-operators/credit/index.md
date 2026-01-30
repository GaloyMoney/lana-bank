---
id: index
title: Credit Module
sidebar_position: 1
---

# Credit Module Lifecycle

![Credit Facility Lifecycle](/img/architecture/credit-lifecycle-1.png)

```mermaid
flowchart LR
    %% Loan = Credit Facility (n1) + Disbursal (S_D)
    subgraph Loan["Loan"]
    direction LR
        n1["Credit Facility <br/>&lt;InterestAccrualCycle&gt;"]

        subgraph S_D["Disbursal"]
        direction LR
            d1["Disbursal 1"]:::small
            d2["Disbursal 2"]:::small
        end
    end

    subgraph S_O["Obligation"]
    direction LR
        o1["Obligation 1"]:::small
        o2["Obligation 2"]:::small
        o3["Obligation 3"]:::small
    end

    subgraph S_R["."]
    direction LR
        subgraph S_R1["Payment 1"]
        direction LR
            r1["PaymentAllocation 1"]:::small
            r2["PaymentAllocation 2"]:::small
        end
        subgraph S_R2["Payment 2"]
        direction LR
            r3["PaymentAllocation 3"]:::small
        end
        r3["PaymentAllocation 3"]:::small
    end

    n1 --> S_D --> S_O

    o1 --> r1
    o2 --> r2
    o2 --> r3
    o3 --> r3

    classDef small stroke:#999,stroke-width:1px;
    style Loan stroke:#666,stroke-width:2px,stroke-dasharray:6 3,fill:none;
```

> A [`CreditFacility`](./facility) advances funds to a borrower through one or more [`Disbursals`](./disbursal).
  Each disbursal creates corresponding [`Obligations`](./obligation) (for *Principal* or any *Accrued Interest*) that the borrower must repay.
  When the borrower makes a [`Payment`](./payment), it is allocated to specific obligations via [`PaymentAllocation`](./payment#payment-allocation) records.
  [`Terms`](./terms) define the interest rates, schedules and other rules that govern the facility and its obligations.
  Once every obligation is fully satisfied, the credit facility is automatically closed.
