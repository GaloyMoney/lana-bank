resource "cala_balance_sheet" "lava" {
  journal_id = cala_journal.journal.id
}

# Schedule 1: Equity
resource "cala_account_set_member_account" "shareholder_equity_in_balance_sheet" {
  account_set_id    = cala_balance_sheet.lava.schedule1_account_set_id
  member_account_id = cala_account.bank_shareholder_equity.id
}

# Schedule 2: ...
# ...

# Schedule 3: ...
# ...

# Schedule 4: ...
# ...

# Schedule 5: ...
resource "cala_account_set_member_account_set" "customer_checking_control_in_balance_sheet" {
  account_set_id        = cala_balance_sheet.lava.schedule5_account_set_id
  member_account_set_id = cala_account_set.customer_checking_control.id
}

resource "cala_account_set_member_account_set" "interest_revenue_control_in_balance_sheet" {
  account_set_id        = cala_balance_sheet.lava.schedule5_account_set_id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}

# Schedule 6: ...
resource "cala_account_set_member_account" "bank_reserve_in_balance_sheet" {
  account_set_id    = cala_balance_sheet.lava.schedule6_account_set_id
  member_account_id = cala_account.bank_reserve.id
}

# Schedule 7: ...
resource "cala_account_set_member_account" "bank_deposits_in_balance_sheet" {
  account_set_id    = cala_balance_sheet.lava.schedule7_account_set_id
  member_account_id = cala_bitfinex_integration.bank_deposits.omnibus_account_id
}

# Schedule 8: ...
# ...

# Schedule 9: ...
resource "cala_account_set_member_account_set" "loans_receivable_control_in_balance_sheet" {
  account_set_id        = cala_balance_sheet.lava.schedule9_account_set_id
  member_account_set_id = cala_account_set.loans_receivable_control.id
}

# Schedule 10: ...
# ...

# Schedule 11: ...
# ...

# Schedule 12: ...
# ...
