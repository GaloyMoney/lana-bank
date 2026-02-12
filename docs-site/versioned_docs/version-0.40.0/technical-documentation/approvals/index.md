---
id: index
title: Governance System
sidebar_position: 1
---

# Governance and Approval System

The governance system provides a structured approval mechanism for critical financial operations requiring multi-party authorization before execution.

```mermaid
graph LR
    subgraph DomainService["Domain Service Internal Structure"]
        CMD["Command"] -->|"validates & executes"| AGG["Aggregate Root<br/>(es-entity)"]
        AGG -->|"emits"| EVT["Domain Events"]
        EVT -->|"persists to"| REPO["Repository"]
        EVT -->|"publishes via"| OUTBOX["Outbox Publisher"]
    end

    subgraph Infrastructure
        REPO -->|"persists"| PG[("PostgreSQL<br/>Event Store")]
        OUTBOX -->|"writes"| OE[("outbox_events<br/>Table")]
    end
```

## Purpose

The system acts as a guardian for high-risk actions:
- Credit facility proposals
- Loan disbursements
- Customer withdrawals

## System Architecture

```mermaid
graph TD
    subgraph GOV["Governance System"]
        POL["Policy Definitions"]
        PROC["Approval Processes"]
        COM["Committee Registry"]
    end
    GOV --> EVT["Event System<br/>(Outbox Pattern)"]
```

## Approval Process Types

| Process Type | Constant | Purpose |
|--------------|----------|---------|
| Credit Facility Proposal | `APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS` | Approve new applications |
| Disbursement | `APPROVE_DISBURSAL_PROCESS` | Approve disbursements |
| Withdrawal | `APPROVE_WITHDRAWAL_PROCESS` | Approve customer withdrawals |

## Approval Flow Lifecycle

```mermaid
graph TD
    INIT["Initiated"] --> PROC["In Process"]
    PROC --> APPR["Approved"]
    PROC --> REJ["Rejected"]
```

### Process Status

| Status | Description |
|--------|-------------|
| PENDING | Process initiated, awaiting review |
| IN_REVIEW | Process under committee review |
| APPROVED | Process approved |
| DENIED | Process rejected |

## System Components

### Policy Definitions

Policies define rules for each approval type:
- Approval thresholds
- Responsible committees
- Quorum rules

### Committee Registry

Manages approval committees:
- Committee members
- Roles and permissions
- Decision history

### Approval Processes

Executes the approval flow:
- Requirements validation
- Vote collection
- Decision execution

## Related Documentation

- [Committee Configuration](committees) - Managing approval committees
- [Approval Policies](policies) - Policy configuration

