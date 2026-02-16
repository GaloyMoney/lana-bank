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

## Admin Panel Walkthrough: User and Role Management

Governance operations depend on correct user-role assignments. Lana uses role-based access control
where roles map to permission sets, and effective permissions are the union across assigned roles.

**Step 1.** Open the users list.

![Users list](/img/screenshots/current/en/user.cy.ts/1_users_list.png)

**Step 2.** Click **Create**.

![Create user button](/img/screenshots/current/en/user.cy.ts/2_click_create_button.png)

**Step 3.** Enter user email.

![Enter user email](/img/screenshots/current/en/user.cy.ts/3_enter_email.png)

**Step 4.** Select initial role (example: admin role assignment).

![Assign admin role](/img/screenshots/current/en/user.cy.ts/4_assign_admin_role.png)

**Step 5.** Submit user creation.

![Submit user creation](/img/screenshots/current/en/user.cy.ts/5_submit_creation.png)

**Step 6.** Verify creation success.

![Verify user created](/img/screenshots/current/en/user.cy.ts/6_verify_creation.png)

**Step 7.** Confirm user appears in list.

![User in list](/img/screenshots/current/en/user.cy.ts/7_view_in_list.png)

**Step 8.** Open role-management for the user.

![Manage user roles](/img/screenshots/current/en/user.cy.ts/8_manage_roles.png)

**Step 9.** Update role set/permissions.

![Update user roles](/img/screenshots/current/en/user.cy.ts/9_update_roles.png)

**Step 10.** Verify role update success.

![Verify role update](/img/screenshots/current/en/user.cy.ts/10_verify_update.png)

