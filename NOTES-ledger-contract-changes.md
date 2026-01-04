
# Ledger Contract — Credit Facility (Final Naming)

## Purpose

This ledger models a credit facility by **strictly separating capacity, obligation, accounting recognition, and cash/application**.

Each account belongs to **exactly one dimension**.  
Only **disbursement** couples capacity and obligation.

---

## Dimensions & Sources of Truth

| Dimension | Source of Truth |
|---------|-----------------|
| Capacity | `facility` |
| Obligation | `uncovered_outstanding` |
| Obligation explainers | `cumulative_interest_added_to_obligations`, `cumulative_payments_made` |
| Accounting receivables | `disbursed_receivable`, `interest_receivable` |
| Income | Interest income account |
| Cash & application | Deposit / cash + facility payment account |

---

## Account Definitions

### 1. Capacity

### `facility`

**Dimension**: Capacity / authorization  
**Nature**: Constraint account

**Tracks**
- Remaining drawable facility capacity

**Moves when**
- Disbursement (↓)
- Limit increase / decrease
- Disbursement reversal

**Does NOT move when**
- Interest accrual
- Payments
- Allocation
- Write-offs

**Authoritative for**
> “Can the borrower draw more?”

---

### 2. Obligation (economic truth)

### `uncovered_outstanding`

**Dimension**: Obligation  
**Nature**: Balance-sheet control

**Tracks**
- Total amount owed by the borrower:
  - principal
  - interest added to obligation
  - future fees / penalties
  - net of payments received

**Moves when**
- Disbursement (↑)
- Interest added to obligation (↑)
- Payment receipt (↓)
- Forgiveness / write-downs (↓)

**Does NOT move when**
- Income recognition
- Allocation
- Cash receipt alone
- Capacity changes

**Authoritative for**
> “How much is owed right now?”

---

### 3. Obligation explainers (monotonic)

### `cumulative_interest_added_to_obligations`

**Dimension**: Obligation explainer  
**Nature**: Monotonic counter (non-clearing)

**Tracks**
- Lifetime interest that increased borrower obligation

**Moves when**
- Interest is **added to obligation**

**Does NOT move when**
- Interest is earned but deferred
- Interest is forgiven
- Payments or allocations

**Notes**
- Not income
- Not receivable
- Explains *why* `uncovered_outstanding` grew

---

### `cumulative_payments_made`

**Dimension**: Obligation explainer  
**Nature**: Monotonic counter (non-clearing)

**Tracks**
- Gross cash collected from borrowers

**Moves when**
- Payment is received

**Does NOT move when**
- Allocation
- Write-offs
- Forgiveness

**Notes**
- Has **no mirror**
- Analytics / KPI only
- Not authoritative for balances

---

### 4. Accounting receivables (typed)

### `disbursed_receivable`

**Dimension**: Accounting  
**Nature**: Balance-sheet receivable

**Tracks**
- Outstanding principal receivable

**Moves when**
- Disbursement (↑)
- Payment allocation to principal (↓)
- Write-off / forgiveness (↓)

**Does NOT move when**
- Cash receipt (pre-allocation)
- Interest accrual

---

### `interest_receivable`

**Dimension**: Accounting  
**Nature**: Balance-sheet receivable

**Tracks**
- Earned but unpaid interest

**Moves when**
- Interest is earned (↑)
- Payment allocation to interest (↓)
- Write-off / forgiveness (↓)

**Does NOT move when**
- Interest is capitalized
- Cash receipt alone

---

## Canonical Events & Postings

### Disbursement
Consumes capacity and creates obligation.

```
Dr facility
Cr uncovered_outstanding

Dr disbursed_receivable
Cr deposit
```

---

### Interest Accrual (earned + owed)

```
Dr cumulative_interest_added_to_obligations
Cr uncovered_outstanding

Dr interest_receivable
Cr interest_income
```

*(The two pairs are independent facts.)*

---

### Payment Receipt (pre-allocation)

```
Dr uncovered_outstanding
Cr cumulative_payments_made

Dr deposit
Cr facility_payment
```

---

### Payment Allocation

```
Dr facility_payment
Cr interest_receivable
Cr disbursed_receivable
```

(No cash, no obligation movement.)

---

## Invariants (Must Hold)

### Capacity
```
facility ≥ 0
```

---

### Obligation
```
uncovered_outstanding
= disbursed_receivable
+ interest added to obligation
+ fees
− payments
− forgiveness
```

---

### Allocation
```
facility_payment
= sum(unallocated payment amounts)
```

---

## Forbidden Patterns

- Treating `facility` as a mirror of `uncovered_outstanding`
- Using `cumulative_interest_added_to_obligations` as income
- Reducing monotonic counters
- Allocating payments before reducing `uncovered_outstanding`

---

## Chart of Accounts Placement (Guidance for ncf-01.csv)

### Balance Sheet → Assets
- `disbursed_receivable` → Loans / Credit Receivables / Principal
- `interest_receivable` → Accrued Interest Receivable
- `deposit` → Cash / Customer Deposits (depending on perspective)

### Balance Sheet → Obligation Control
- `uncovered_outstanding`  
  → Loan Obligations Outstanding  
  (Often under Credit Portfolio Control)

### Memo / Statistical / Control Section
- `cumulative_interest_added_to_obligations`
- `cumulative_payments_made`

### Off-Balance-Sheet / Capacity Controls
- `facility`  
  → Credit Commitments / Available Facility

---

## Final Takeaway

This naming and structure:
- cleanly separates capacity, obligation, income, and cash
- supports fees, restructures, capitalization, and write-offs
- is explainable to both accountants and engineers
- is safe to extend without breaking invariants
