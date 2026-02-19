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

