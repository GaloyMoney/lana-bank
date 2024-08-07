resource "cala_account_set" "balance_sheet" {
  id                  = "00000000-0000-0000-0000-100000000003"
  journal_id          = cala_journal.journal.id
  name                = "Balance Sheet"
  normal_balance_type = "DEBIT"
}

# ASSETS
resource "cala_account_set_member_account_set" "assets_in_balance_sheet" {
  account_set_id        = cala_account_set.balance_sheet.id
  member_account_set_id = cala_account_set.assets.id
}

# LIABILITIES
resource "cala_account_set_member_account_set" "liabilities_in_balance_sheet" {
  account_set_id        = cala_account_set.balance_sheet.id
  member_account_set_id = cala_account_set.liabilities.id
}

# EQUITY
resource "random_uuid" "equity_for_balance_sheet" {}
resource "cala_account_set" "equity_for_balance_sheet" {
  id                  = random_uuid.equity_for_balance_sheet.result
  journal_id          = cala_journal.journal.id
  name                = "Equity"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "equity_in_balance_sheet" {
  account_set_id        = cala_account_set.balance_sheet.id
  member_account_set_id = cala_account_set.equity_for_balance_sheet.id
}

# EQUITY: Members
resource "cala_account_set_member_account_set" "net_income_in_equity" {
  account_set_id        = cala_account_set.equity_for_balance_sheet.id
  member_account_set_id = cala_account_set.net_income.id
}

resource "cala_account_set_member_account" "bank_shareholder_equity_in_equity" {
  account_set_id    = cala_account_set.equity_for_balance_sheet.id
  member_account_id = cala_account.bank_shareholder_equity.id
}
