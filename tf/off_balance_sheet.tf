resource "random_uuid" "collateral_deposits_control" {}
resource "cala_account_set" "collateral_deposits_control" {
  id                  = random_uuid.collateral_deposits_control.result
  journal_id          = cala_journal.journal.id
  name                = "Off-Balance-Sheet Deposits For Collateral From Users Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "collateral_deposits_control_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.collateral_deposits_control.id
}

resource "cala_account_set" "user_collateral_control" {
  id                  = "00000000-0000-0000-0000-210000000001"
  journal_id          = cala_journal.journal.id
  name                = "User Collateral Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "user_collateral_control_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_collateral_control.id
}

resource "cala_account_set" "loans_collateral_control" {
  id                  = "00000000-0000-0000-0000-210000000002"
  journal_id          = cala_journal.journal.id
  name                = "Loans Collateral Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "loans_collateral_control_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.loans_collateral_control.id
}

