# Trial Balance Account Set
resource "cala_account_set" "trial_balance" {
  id                  = "00000000-0000-0000-0000-110000000000"
  journal_id          = cala_journal.journal.id
  name                = "Trial Balance"
  normal_balance_type = "DEBIT"
}

# GL "Control Account" Account Sets
resource "cala_account_set" "user_deposits_control" {
  id                  = "00000000-0000-0000-0000-110000000001"
  journal_id          = cala_journal.journal.id
  name                = "Bank Deposits for User Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "gl_user_deposits" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_deposits_control.id
}
resource "cala_account_set_member_account" "gl_bank_deposits" {
  account_set_id    = cala_account_set.user_deposits_control.id
  member_account_id = cala_bitfinex_integration.bank_deposit.omnibus_account_id
}

resource "cala_account_set" "user_collateral_deposits_control" {
  id                  = "00000000-0000-0000-0000-110000000002"
  journal_id          = cala_journal.journal.id
  name                = "Bank Collateral for User Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "gl_user_collateral_deposits" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_collateral_deposits_control.id
}
resource "cala_account_set_member_account" "gl_bank_collateral_deposits" {
  account_set_id    = cala_account_set.user_collateral_deposits_control.id
  member_account_id = cala_bitfinex_integration.off_balance_sheet.omnibus_account_id
}

resource "cala_account_set" "user_checking_control" {
  id                  = "00000000-0000-0000-0000-110000000003"
  journal_id          = cala_journal.journal.id
  name                = "User Checking Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "gl_user_checking" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_checking_control.id
}

resource "cala_account_set" "user_collateral_control" {
  id                  = "00000000-0000-0000-0000-110000000004"
  journal_id          = cala_journal.journal.id
  name                = "User Collateral Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "gl_user_collateral" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_collateral_control.id
}

resource "cala_account_set" "loans_control" {
  id                  = "00000000-0000-0000-0000-110000000005"
  journal_id          = cala_journal.journal.id
  name                = "Loans Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "gl_loans" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.loans_control.id
}

resource "cala_account_set" "loans_collateral_control" {
  id                  = "00000000-0000-0000-0000-110000000006"
  journal_id          = cala_journal.journal.id
  name                = "Loans Collateral Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "gl_loans_collateral" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.loans_collateral_control.id
}

resource "cala_account_set" "interest_revenue_control" {
  id                  = "00000000-0000-0000-0000-110000000007"
  journal_id          = cala_journal.journal.id
  name                = "Interest Revenue Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "gl_interest_revenue" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}

resource "cala_account_set" "penalty_revenue_control" {
  id                  = "00000000-0000-0000-0000-110000000008"
  journal_id          = cala_journal.journal.id
  name                = "Penalty Revenue Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "gl_penalty_revenue" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.penalty_revenue_control.id
}

# GL Accounts
resource "cala_account_set_member_account" "gl_bank_shareholder_equity" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_shareholder_equity.id
}

resource "cala_account_set_member_account" "gl_bank_reserve" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_reserve.id
}
