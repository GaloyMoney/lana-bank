resource "cala_balance_sheet" "lava" {
  journal_id = cala_journal.journal.id
}

resource "cala_account_set" "shareholder_equity" {
  id         = "00000000-0000-0000-0000-100000000001"
  journal_id = cala_journal.journal.id
  name       = "Shareholder Equity"
}

resource "cala_account_set_member_account_set" "shareholder_equity" {
  account_set_id        = cala_balance_sheet.lava.schedule1_account_set_id
  member_account_set_id = cala_account_set.shareholder_equity.id
}

resource "cala_account_set" "user_checking" {
  id                  = "00000000-0000-0000-0000-500000000002"
  journal_id          = cala_journal.journal.id
  name                = "User Checking"
  normal_balance_type = "CREDIT"
}

resource "cala_account_set_member_account_set" "user_checking_member" {
  account_set_id        = cala_balance_sheet.lava.schedule5_account_set_id
  member_account_set_id = cala_account_set.user_checking.id
}

resource "cala_account_set" "off_balance_sheet_user_deposits" {
  id         = "10000000-0000-0000-0000-300000000000"
  journal_id = cala_journal.journal.id
  name       = "User Deposits"
}

resource "cala_account_set_member_account" "bfx_deposits" {
  account_set_id    = cala_balance_sheet.lava.schedule7_account_set_id
  member_account_id = cala_bitfinex_integration.bank_deposit.omnibus_account_id
}

resource "cala_account_set" "loans" {
  id                  = "00000000-0000-0000-0000-900000000001"
  journal_id          = cala_journal.journal.id
  name                = "Loans"
  normal_balance_type = "DEBIT"
}

resource "cala_account_set_member_account_set" "loans" {
  account_set_id        = cala_balance_sheet.lava.schedule9_account_set_id
  member_account_set_id = cala_account_set.loans.id
}

resource "cala_account_set" "interest_revenue" {
  id         = "00000000-0000-0000-0000-500000000001"
  journal_id = cala_journal.journal.id
  name       = "Interest Revenue"
}

resource "cala_account_set_member_account_set" "interest_revenue" {
  account_set_id        = cala_balance_sheet.lava.schedule5_account_set_id
  member_account_set_id = cala_account_set.interest_revenue.id
}

resource "cala_account_set" "penalty_revenue" {
  id         = "00000000-0000-0000-0000-500000000003"
  journal_id = cala_journal.journal.id
  name       = "Penalty Revenue"
}

resource "cala_account_set_member_account_set" "penalty_revenue" {
  account_set_id        = cala_balance_sheet.lava.schedule5_account_set_id
  member_account_set_id = cala_account_set.penalty_revenue.id
}

resource "cala_account_set" "reserves" {
  id         = "00000000-0000-0000-0000-600000000001"
  journal_id = cala_journal.journal.id
  name       = "Bank Reserves"
}

resource "cala_account_set_member_account_set" "reserves" {
  account_set_id        = cala_balance_sheet.lava.schedule6_account_set_id
  member_account_set_id = cala_account_set.reserves.id
}
