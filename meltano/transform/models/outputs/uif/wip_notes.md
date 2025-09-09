I'm trying to understand what are all the tables in lana's backend I need to look at to find transactions.

These are my first candidates:
- `core_deposit_accounts` ( + `_events` + `_rollup` )
- `core_deposits` ( + `_events` + `_rollup` )
- `core_withdrawals` ( + `_events` + `_rollup` )


I would like to perform this flow and find the created data along in the DB:
- Create a customer (`alice@alice.com`)
  - DB: find his ID
  - DB: check that it got a deposit account created
- Make a deposit for his deposit account of 999USD
- Make a withdrawal of 333USD




## Q&A

- Where do we get an integer ID for the accounts to be reported in the transactions?
    - `core_deposit_accounts` has a public id for each account.
- What state should we report?
    - Deposits 

- Reversability
    - Which transactions can be reversed? What are the time limits? How does it get managed accounting wise?