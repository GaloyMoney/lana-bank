## `FiscalYear`

### Initialization

No transactions can be posted until the first `FiscalYear` entity has been initialized. An account set metadata update made to the root account set of the `Chart` the `FiscalYear` relates to enables velocity controls to be satisfied.

There are 2 ways to initialize the first `FiscalYear`:

- On initial startup if `accounting_init` has a `chart_of_accounts_opening_date` set in YAML.

- Via GraphQL mutation.

### Closing Months of a `FiscalYear`

Monthly closes lock the entire ledger against transactions with an `effective_date` before `month_closed_as_of`. To execute this entity command, the precondition that the month has past (according to `crate::time::now()`) must be satisfied. The command applies to the oldest, unclosed month of the `FiscalYear`.

### Closing the `FiscalYear`

If the last month of a `FiscalYear` has been closed, the `FiscalYear` lifecycle can be completed. This posts a transaction to the ledger, with an `effective_date` set to the `FiscalYear`'s `closed_as_of`. 

The accounting significance of this transaction is to transfer net income for the `FiscalYear` from the `ProfitAndLossStatement` to the `BalanceSheet`.

### Open the next `FiscalYear`

Required as an explicit action.