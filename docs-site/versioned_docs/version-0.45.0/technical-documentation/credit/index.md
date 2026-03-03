---
id: index
title: Credit Module
sidebar_position: 1
---

# Credit Module

The credit module manages the full lifecycle of Bitcoin-backed loans in Lana. It handles everything from initial proposal creation through collateralization, disbursement of funds, interest accrual, repayment tracking, and eventual facility closure. All credit operations are secured by Bitcoin collateral, with continuous monitoring of collateral-to-loan value ratios to protect the bank against market risk.

## How Credit Works in Lana

Lana provides credit facilities where a customer borrows USD backed by Bitcoin collateral. The fundamental flow is:

1. A bank operator creates a **proposal** defining the loan amount and terms for a specific customer.
2. The customer accepts the proposal, and it goes through **governance approval** (committee voting or auto-approval).
3. Upon approval, the customer must **post Bitcoin collateral** sufficient to meet the initial collateral-to-loan value (CVL) ratio.
4. Once collateral requirements are met, the facility **activates** and the customer can draw funds through **disbursals**.
5. Interest **accrues** daily on outstanding principal and is consolidated into payable **obligations** monthly.
6. The customer makes **payments** that are automatically allocated to outstanding obligations in priority order.
7. When all obligations are fully paid, the facility **closes** and collateral can be returned.

Throughout this lifecycle, the system continuously monitors the BTC/USD exchange rate and recalculates the CVL. If the collateral value drops below safety thresholds, the system can trigger margin calls or initiate liquidation proceedings.

## Entity Relationships

```mermaid
flowchart LR
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

The credit module is built around five core entities:

- A [**Credit Facility**](./facility) is the lending agreement that defines the credit limit, terms, and collateral requirements. It advances funds to a borrower through one or more disbursals.
- A [**Disbursal**](./disbursal) represents a specific drawdown of funds from the facility to the customer. Each disbursal goes through its own approval process and, when confirmed, creates a principal obligation.
- An [**Obligation**](./obligation) tracks an individual amount owed by the borrower, either for principal (from a disbursal) or interest (from an accrual cycle). Obligations follow a time-driven lifecycle from not-yet-due through due, overdue, and potentially defaulted.
- A [**Payment**](./payment) captures funds remitted by the borrower. Each payment is automatically broken down into payment allocations that settle specific obligations in priority order.
- [**Terms**](./terms) define the interest rates, fee schedules, timing intervals, and collateral thresholds that govern the facility. Terms are set at proposal time and remain fixed for the life of the facility.

## Collateral and Risk Management

Because Lana issues USD loans backed by Bitcoin, the relationship between collateral value and loan exposure is central to risk management. The system tracks three CVL (Collateral Value to Loan) thresholds defined in the facility terms:

| Threshold | Purpose |
|-----------|---------|
| **Initial CVL** | The minimum collateral ratio required to activate the facility. The customer must post enough BTC so that its USD value exceeds this ratio relative to the facility amount. |
| **Margin Call CVL** | A safety buffer below the initial threshold. If the CVL drops below this level due to BTC price declines, the system flags the facility for a margin call, alerting operators and the borrower that additional collateral may be needed. New disbursals are also blocked if they would push the CVL below this level. |
| **Liquidation CVL** | The critical floor. If the CVL falls below this threshold, the system initiates a liquidation process where the bank can sell collateral to recover the outstanding debt. |

These thresholds must maintain a strict hierarchy: Initial CVL > Margin Call CVL > Liquidation CVL. The system enforces this at proposal creation time.

The CVL is continuously recalculated as the BTC/USD price changes, as collateral is deposited or withdrawn, and as the outstanding loan balance changes through disbursals and payments. A hysteresis buffer prevents rapid oscillation between states when the CVL hovers near a threshold boundary.

## Facility Lifecycle Overview

```mermaid
stateDiagram-v2
    [*] --> Proposal: Operator creates proposal
    Proposal --> PendingCustomerApproval: Proposal submitted
    PendingCustomerApproval --> PendingApproval: Customer accepts
    PendingCustomerApproval --> CustomerDenied: Customer rejects
    PendingApproval --> Approved: Committee approves
    PendingApproval --> Denied: Committee denies
    Approved --> PendingCollateralization: Pending facility created
    PendingCollateralization --> Completed: CVL >= initial_cvl
    Completed --> Active: Facility activated
    Active --> Closed: All obligations paid
    CustomerDenied --> [*]
    Denied --> [*]
    Closed --> [*]
```

For detailed information on each stage, see [Credit Facilities](./facility).

## Interest Lifecycle

Interest accrual uses a two-level timing system. Daily accrual jobs record interest in the ledger as it is earned. Monthly cycle jobs consolidate those accruals into payable interest obligations. This design satisfies both accounting requirements (revenue recognized as earned) and borrower experience (predictable monthly billing).

For the full mechanics, see [Interest Processing](./interest-process).

## Module Pages

| Page | Description |
|------|-------------|
| [Credit Facilities](./facility) | Creating proposals, approval process, collateralization, activation, and facility states |
| [Disbursals](./disbursal) | Drawing funds from active facilities, approval flow, and settlement |
| [Obligations](./obligation) | Debt tracking, obligation types, lifecycle states, and timing parameters |
| [Payments](./payment) | Payment processing, allocation priority rules, and accounting impact |
| [Terms](./terms) | Interest rates, fee schedules, timing intervals, CVL thresholds, and terms templates |
| [Interest Processing](./interest-process) | Daily accrual, monthly cycles, obligation creation, and ledger entries |
| [Ledger](./ledger.md) | Overview of account sets and transaction templates |
