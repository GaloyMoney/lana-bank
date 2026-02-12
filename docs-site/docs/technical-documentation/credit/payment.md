---
id: payment
title: Payment
sidebar_position: 5
---

# Payment

Captures funds remitted by the borrower toward a facility.
Each payment breaks down into one or more allocations that settle specific obligations in priority order.
Events emitted from payments update repayment plans, balances and close obligations once fully covered.

## Payment Allocation

When a payment is received, it is allocated to outstanding obligations based on priority rules. Each `PaymentAllocation` record links a portion of the payment to a specific obligation, reducing its outstanding balance.
