## Interest Process

Interest on a credit facility accrues periodically and is captured as new obligations.
The process is orchestrated by a pair of jobs:

1. **Interest Accrual Job (`interest-accrual`)** – confirms accrued interest for the current period. 
   It records a ledger entry for the accrued amount and reschedules itself for the next
   accrual period. After the final period in a cycle, it spawns the cycle job.

2. **Interest Accrual Cycle Job (`interest-accrual-cycle`)** – posts the confirmed
   accruals for the completed cycle. This job creates an `Obligation` of type
   *Interest* and schedules the first accrual of the next cycle.

Each interest obligation is linked back to the facility so that when a borrower makes a
`Payment`, `PaymentAllocation` records can reduce the outstanding interest balance.
