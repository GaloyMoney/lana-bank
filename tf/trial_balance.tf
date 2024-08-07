resource "cala_account_set" "trial_balance" {
  id                  = "00000000-0000-0000-0000-100000000002"
  journal_id          = cala_journal.journal.id
  name                = "Trial Balance"
  normal_balance_type = "DEBIT"
}

# ASSETS: Members
resource "cala_account_set_member_account" "bank_deposits_omnibus_in_trial_balance" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_deposits_omnibus.id
}


resource "cala_account_set_member_account_set" "loans_principal_receivable_control_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.loans_principal_receivable_control.id
}

resource "cala_account_set_member_account_set" "loans_interest_receivable_control_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.loans_interest_receivable_control.id
}

resource "cala_account_set_member_account" "bank_reserve_in_trial_balance" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_reserve.id
}


# LIABILITIES: Members
resource "cala_account_set_member_account_set" "customer_checking_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.customer_checking_control.id
}


# EQUITY: Members
resource "cala_account_set_member_account" "bank_shareholder_equity_in_trial_balance" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_shareholder_equity.id
}


# REVENUE: Members
resource "cala_account_set_member_account_set" "interest_revenue_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}


# EXPENSES: Members
# <None>
