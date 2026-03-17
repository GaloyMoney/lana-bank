-- Convert amount column from bigint (UsdCents) to jsonb (CurrencyBag)
-- for multi-currency support in deposits and withdrawals.

-- Deposits: convert existing bigint amounts to CurrencyBag JSON format
ALTER TABLE core_deposits
  ALTER COLUMN amount TYPE jsonb
  USING jsonb_build_object('usd', amount);

-- Withdrawals: convert existing bigint amounts to CurrencyBag JSON format
ALTER TABLE core_withdrawals
  ALTER COLUMN amount TYPE jsonb
  USING jsonb_build_object('usd', amount);
