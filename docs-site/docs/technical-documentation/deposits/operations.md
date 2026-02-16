---
id: operations
title: Deposit and Withdrawal Operations
sidebar_position: 2
---

# Deposit and Withdrawal Operations

This document describes deposit and withdrawal operations, including workflows and approval procedures.

## Deposit Operations

### Recording Deposits

Deposits are recorded when external funds are received into a customer account.

```mermaid
graph LR
    RCV["Receive funds"] --> REC["Record deposit"] --> AVL["Funds available"]
```

### Create a Deposit

#### From Admin Panel

1. Navigate to **Customers** > select customer
2. Go to deposit account
3. Click **Record Deposit**
4. Complete:
   - Amount in USD
   - External reference
5. Confirm operation

## Withdrawal Operations

### Withdrawal Flow

Withdrawals require an approval process before execution.

```mermaid
graph TD
    REQ["Withdrawal request"] --> APPR["Approval required"]
    APPR --> EXEC["Execute withdrawal"]
    APPR --> REJ["Rejected<br/>(optional)"]
```

### Initiate a Withdrawal

#### From Admin Panel

1. Navigate to **Customers** > select customer
2. Go to deposit account
3. Click **Initiate Withdrawal**
4. Complete:
   - Amount in USD
   - External reference
5. Withdrawal enters approval process

### Withdrawal Status

| Status | Description |
|--------|-------------|
| PENDING_APPROVAL | Withdrawal pending approval |
| APPROVED | Withdrawal approved |
| CONFIRMED | Withdrawal executed and confirmed |
| DENIED | Withdrawal rejected |
| CANCELLED | Withdrawal cancelled |

## Approval Process

Withdrawals are subject to the governance system with process type `APPROVE_WITHDRAWAL_PROCESS`.

### Approve a Withdrawal

1. Navigate to **Pending Approvals**
2. Select withdrawal to approve
3. Review details:
   - Customer
   - Amount
   - Available balance
4. Click **Approve** or **Reject**

## Accounting Integration

### Deposit Entries

When a deposit is recorded:

| Account | Debit | Credit |
|---------|-------|--------|
| Cash (Asset) | X | |
| Customer Deposits (Liability) | | X |

### Withdrawal Entries

When a withdrawal is confirmed:

| Account | Debit | Credit |
|---------|-------|--------|
| Customer Deposits (Liability) | X | |
| Cash (Asset) | | X |

## Permissions Required

| Operation | Permission |
|-----------|---------|
| Record deposit | DEPOSIT_CREATE |
| View deposits | DEPOSIT_READ |
| Initiate withdrawal | WITHDRAWAL_CREATE |
| Approve withdrawal | WITHDRAWAL_APPROVE |
| Confirm withdrawal | WITHDRAWAL_CONFIRM |

## Admin Panel Walkthrough: Deposits and Withdrawals

This flow shows operational creation and management of deposits and withdrawals.

### A) Create a deposit

**Step 1.** Click global **Create**.

![Open create menu](/img/screenshots/current/en/transactions.cy.ts/1_deposit_create_button.png)

**Step 2.** Select **Create Deposit**.

![Select create deposit](/img/screenshots/current/en/transactions.cy.ts/2_deposit_select.png)

**Step 3.** Enter deposit amount.

![Enter deposit amount](/img/screenshots/current/en/transactions.cy.ts/3_deposit_enter_amount.png)

**Step 4.** Submit.

![Submit deposit](/img/screenshots/current/en/transactions.cy.ts/4_deposit_submit.png)

**Step 5.** Confirm success message.

![Deposit success](/img/screenshots/current/en/transactions.cy.ts/5_deposit_success.png)

**Step 6.** Verify deposit in deposit list.

![Deposit appears in list](/img/screenshots/current/en/transactions.cy.ts/6_deposit_in_list.png)

**Step 7.** Verify deposit in customer transaction history.

![Deposit in transaction history](/img/screenshots/current/en/transactions.cy.ts/7_deposit_in_transactions.png)

### B) Create a withdrawal

**Step 8.** Click **Create** for withdrawal initiation.

![Open withdrawal create](/img/screenshots/current/en/transactions.cy.ts/8_withdrawal_create_button.png)

**Step 9.** Select **Create Withdrawal**.

![Select create withdrawal](/img/screenshots/current/en/transactions.cy.ts/9_withdrawal_select.png)

**Step 10.** Enter withdrawal amount.

![Enter withdrawal amount](/img/screenshots/current/en/transactions.cy.ts/10_withdrawal_enter_amount.png)

**Step 11.** Submit the request.

![Submit withdrawal](/img/screenshots/current/en/transactions.cy.ts/11_withdrawal_submit.png)

**Step 12.** Verify withdrawal appears in withdrawal list.

![Withdrawal in list](/img/screenshots/current/en/transactions.cy.ts/12_withdrawal_in_list.png)

**Step 13.** Verify withdrawal appears in customer transactions.

![Withdrawal in transaction history](/img/screenshots/current/en/transactions.cy.ts/13_withdrawal_in_transactions.png)

### C) Manage withdrawal outcome

#### Cancel a pending withdrawal

**Step 14.** Click **Cancel**.

![Cancel withdrawal button](/img/screenshots/current/en/transactions.cy.ts/14_withdrawal_cancel_button.png)

**Step 15.** Confirm cancellation.

![Confirm cancellation](/img/screenshots/current/en/transactions.cy.ts/15_withdrawal_cancel_confirm.png)

**Step 16.** Verify status becomes cancelled.

![Cancelled status](/img/screenshots/current/en/transactions.cy.ts/16_withdrawal_cancelled_status.png)

#### Approve a pending withdrawal

**Step 17.** Click **Approve**.

![Approve withdrawal button](/img/screenshots/current/en/transactions.cy.ts/17_withdrawal_approve_button.png)

**Step 18.** Confirm approval.

![Confirm approval](/img/screenshots/current/en/transactions.cy.ts/18_withdrawal_approve_confirm.png)

**Step 19.** Verify approved/confirmed status.

![Approved withdrawal status](/img/screenshots/current/en/transactions.cy.ts/19_withdrawal_approved_status.png)

