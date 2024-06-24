Bfx Integrations:

- omnibus account set for all debit accounts created
- withdrawal debit account created to be credited and set negative

Create address-backed user account:

- new "bank deposit" debit account USER.123 created
- account added to integration omnibus debit account
- credit account supplied as input

Record deposit:

- Debit bank deposit
- Credit user checking account

(Missing):

- Debit bank deposit omnibus (helps with reasoning things from GL perspective)
- Credit user bank deposit

Withdraw from account:

- supplied user (credit) checking account debited
- withdrawal (debit) account credited into negative (should be debit omnibus supplied to integration)

Potential TODOs:

- supply omnibus debit account instead of creating omnibus "withdrawal" account
- rename "debitAccountId" in WithdrawalExecute mutation
