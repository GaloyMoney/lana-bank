resource "random_uuid" "bank_shareholder_equity" {}
resource "cala_account" "bank_shareholder_equity" {
  id                  = random_uuid.bank_shareholder_equity.result
  name                = "Bank Shareholder Equity"
  code                = "BANK.SHAREHOLDER_EQUITY"
  normal_balance_type = "CREDIT"
}

resource "cala_account_set_member_account" "bank_shareholder_equity" {
  account_set_id    = cala_account_set.shareholder_equity.id
  member_account_id = cala_account.bank_shareholder_equity.id
}

resource "random_uuid" "bank_reserve" {}
resource "cala_account" "bank_reserve" {
  id                  = random_uuid.bank_reserve.result
  name                = "Bank Reserve from Shareholders"
  code                = "BANK.RESERVE_FROM_SHAREHOLDER"
  normal_balance_type = "DEBIT"
}

resource "cala_account_set_member_account" "bank_reserve" {
  account_set_id    = cala_account_set.reserves.id
  member_account_id = cala_account.bank_reserve.id
}
