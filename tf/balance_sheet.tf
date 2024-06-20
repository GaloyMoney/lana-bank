resource "cala_balance_sheet" "lava" {
  journal_id = cala_journal.journal.id
}

resource "cala_account_set" "user_deposits" {
  id         = "00000000-0000-0000-0000-300000000001"
  journal_id = cala_journal.journal.id
  name       = "User Deposits"
}

resource "cala_account_set_member_account_set" "user_deposits_member" {
  account_set_id        = cala_balance_sheet.lava.schedule3_account_set_id
  member_account_set_id = cala_account_set.user_deposits.id
}

resource "cala_account_set" "off_balance_sheet_user_deposits" {
  id         = "10000000-0000-0000-0000-300000000000"
  journal_id = cala_journal.journal.id
  name       = "User Deposits"
}

resource "cala_account_set_member_account_set" "bfx_deposits" {
  account_set_id        = cala_balance_sheet.lava.schedule7_account_set_id
  member_account_set_id = cala_bitfinex_integration.bank_deposit.omnibus_account_set_id
}

resource "cala_account_set" "fixed_term_loans" {
  id         = "00000000-0000-0000-0000-900000000001"
  journal_id = cala_journal.journal.id
  name       = "Fixed term loans"
}

resource "cala_account_set_member_account_set" "loans" {
  account_set_id        = cala_balance_sheet.lava.schedule9_account_set_id
  member_account_set_id = cala_account_set.fixed_term_loans.id
}
