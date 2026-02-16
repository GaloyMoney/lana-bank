---
id: committees
title: Approval Committees
sidebar_position: 2
---

# Approval Committee Configuration

This document describes how to configure and manage approval committees in the governance system.

## Committee Concept

A committee is a group of authorized users who make decisions on specific operations. Each committee has:

- **Members**: Users with voting rights
- **Quorum**: Minimum votes required
- **Process type**: Category of operations it can approve

## Committee Types

### Credit Committee

Responsible for approving:
- Credit facility proposals
- Loan disbursements

### Operations Committee

Responsible for approving:
- Customer withdrawals
- Special operations

## Committee Management

### Create a Committee

#### From Admin Panel

1. Navigate to **Configuration** > **Committees**
2. Click **New Committee**
3. Configure:
   - Committee name
   - Associated process type
   - Required quorum
4. Add members
5. Save configuration

### Add Members

1. Navigate to the committee detail page
2. Click **Add Member**
3. Select the user from the list
4. Save the changes

## Quorum Configuration

Quorum defines the minimum number of votes needed for a decision.

### Quorum Rules

| Configuration | Description |
|---------------|-------------|
| Simple majority | More than 50% of members |
| Unanimity | All members must vote |
| Fixed number | Specific vote count |

## Voting Process

### Voting Flow

```mermaid
graph LR
    SUB["Request submitted"] --> VOTE["Active voting"] --> DEC["Decision reached"]
```

### Cast a Vote

1. Navigate to **Pending Approvals**
2. Select the request
3. Review details
4. Click **Approve** or **Reject**

## Permissions Required

| Operation | Permission |
|-----------|---------|
| Create committee | COMMITTEE_CREATE |
| View committees | COMMITTEE_READ |
| Modify committee | COMMITTEE_UPDATE |
| Delete committee | COMMITTEE_DELETE |
| Cast vote | VOTE_CREATE |

## Admin Panel Walkthrough: Create Committee and Add Members

### 1) Create committee

**Step 1.** Visit committees page.

![Visit committees page](/img/screenshots/current/en/governance.cy.ts/1_step-visit-committees.png)

**Step 2.** Click **Create Committee**.

![Click create committee](/img/screenshots/current/en/governance.cy.ts/2_step-click-create-committee-button.png)

**Step 3.** Enter committee name.

![Fill committee name](/img/screenshots/current/en/governance.cy.ts/3_step-fill-committee-name.png)

**Step 4.** Submit committee creation.

![Submit committee creation](/img/screenshots/current/en/governance.cy.ts/4_step-submit-committee-creation.png)

**Step 5.** Verify success.

![Committee created successfully](/img/screenshots/current/en/governance.cy.ts/5_step-committee-created-successfully.png)

**Step 6.** Confirm committee appears in list.

![Committee list](/img/screenshots/current/en/governance.cy.ts/6_step-view-committees-list.png)

### 2) Add member

**Step 7.** Open committee details.

![Committee details](/img/screenshots/current/en/governance.cy.ts/7_step-visit-committee-details.png)

**Step 8.** Click **Add Member**.

![Add member button](/img/screenshots/current/en/governance.cy.ts/8_step-click-add-member-button.png)

**Step 9.** Select role/member mapping.

![Select admin role](/img/screenshots/current/en/governance.cy.ts/9_step-select-admin-role.png)

**Step 10.** Submit member addition.

![Submit add member](/img/screenshots/current/en/governance.cy.ts/10_step-submit-add-member.png)

**Step 11.** Verify member is added successfully.

![Verify member added](/img/screenshots/current/en/governance.cy.ts/11_step-verify-member-added.png)

