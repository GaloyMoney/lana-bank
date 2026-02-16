---
id: policies
title: Approval Policies
sidebar_position: 3
---

# Approval Policy Configuration

This document describes how to configure policies governing approval processes in the governance system.

## Policy Concept

A policy defines the rules and conditions under which an operation can be approved:

- **Process type**: Operation category
- **Thresholds**: Limits for different approval levels
- **Escalation rules**: When to escalate to higher committees

## Policy Types

### Credit Facility Policy

Defines rules for approving credit proposals:

| Level | Amount | Required Approval |
|-------|--------|-------------------|
| Low | < $10,000 | 1 approver |
| Medium | $10,000 - $100,000 | 2 approvers |
| High | > $100,000 | Full committee |

### Disbursement Policy

Defines rules for approving disbursements:

| Level | Amount | Required Approval |
|-------|--------|-------------------|
| Low | < $5,000 | Automatic |
| Medium | $5,000 - $50,000 | 1 approver |
| High | > $50,000 | 2 approvers |

### Withdrawal Policy

Defines rules for approving withdrawals:

| Level | Amount | Required Approval |
|-------|--------|-------------------|
| Low | < $1,000 | Automatic |
| Medium | $1,000 - $10,000 | 1 approver |
| High | > $10,000 | Operations committee |

## Policy Configuration

### Create a Policy

```graphql
mutation CreateApprovalPolicy($input: ApprovalPolicyCreateInput!) {
  approvalPolicyCreate(input: $input) {
    policy {
      id
      processType
      thresholds {
        level
        amount
        requiredApprovals
      }
    }
  }
}
```

## Escalation Rules

### Escalation Flow

```mermaid
graph LR
    L1["Level 1<br/>(Auto)"] --> L2["Level 2<br/>(Approver)"] --> L3["Level 3<br/>(Committee)"]
```

### Escalation Conditions

| Condition | Action |
|-----------|--------|
| Amount exceeds threshold | Escalate to next level |
| Time exceeded | Notify and escalate |
| Rejected at lower level | Escalate for review |

## Domain Integration

Policies integrate with domain services:

- Credit facilities use `APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS`
- Disbursements use `APPROVE_DISBURSAL_PROCESS`
- Withdrawals use `APPROVE_WITHDRAWAL_PROCESS`

## Permissions Required

| Operation | Permission |
|-----------|---------|
| Create policy | POLICY_CREATE |
| View policies | POLICY_READ |
| Modify policy | POLICY_UPDATE |
| Delete policy | POLICY_DELETE |

## Policy Auditing

All policy modifications are logged in the audit system:

- Who made the change
- What was modified
- When it occurred
- Previous and new values

## Admin Panel Walkthrough: Assign Committee and Resolve Actions

### 1) Assign committee to policy

**Step 12.** Open policies page.

![Visit policies page](/img/screenshots/current/en/governance.cy.ts/12_step-visit-policies-page.png)

**Step 13.** Select a policy.

![Select policy](/img/screenshots/current/en/governance.cy.ts/13_step-select-policy.png)

**Step 14.** Assign committee and threshold.

![Assign committee to policy](/img/screenshots/current/en/governance.cy.ts/14_step-assign-committee-to-policy.png)

**Step 15.** Verify assignment success.

![Verify committee assigned](/img/screenshots/current/en/governance.cy.ts/15_step-verify-committee-assigned.png)

### 2) Review pending actions

**Step 16.** Open actions queue.

![Actions page](/img/screenshots/current/en/governance.cy.ts/16_step-view-actions-page.png)

**Step 17.** Confirm pending request appears.

![Pending withdrawal visible](/img/screenshots/current/en/governance.cy.ts/17_step-verify-pending-withdrawal.png)

### 3) Approve or deny process

**Step 18.** Open request details for decision.

![Withdrawal details for approval](/img/screenshots/current/en/governance.cy.ts/18_step-visit-withdrawal-details.png)

**Step 19.** Click **Approve**.

![Click approve](/img/screenshots/current/en/governance.cy.ts/19_step-click-approve-button.png)

**Step 20.** Verify approval success and state transition.

![Approval success](/img/screenshots/current/en/governance.cy.ts/20_step-verify-approval-success.png)

**Step 21.** Open request for denial flow.

![Visit withdrawal for denial](/img/screenshots/current/en/governance.cy.ts/21_step-visit-withdrawal-for-denial.png)

**Step 22.** Click **Deny** and provide reason.

![Click deny](/img/screenshots/current/en/governance.cy.ts/22_step-click-deny-button.png)

**Step 23.** Verify denial success and terminal status.

![Denial success](/img/screenshots/current/en/governance.cy.ts/23_step-verify-denial-success.png)

