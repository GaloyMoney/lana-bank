# Off-Balance-Sheet "Chart of Accounts" Account Set
resource "cala_account_set" "obs_chart_of_accounts" {
  id         = "10000000-0000-0000-0000-100000000001"
  journal_id = cala_journal.journal.id
  name       = "Off-Balance-Sheet Chart of Accounts"
}

resource "cala_account_set" "obs_trial_balance" {
  id                  = "10000000-0000-0000-0000-100000000002"
  journal_id          = cala_journal.journal.id
  name                = "Off-Balance-Sheet Trial Balance"
  normal_balance_type = "DEBIT"
}


# ASSETS
resource "random_uuid" "obs_assets" {}
resource "cala_account_set" "obs_assets" {
  id                  = random_uuid.obs_assets.result
  journal_id          = cala_journal.journal.id
  name                = "Assets (Off-Balance-Sheet)"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "obs_assets" {
  account_set_id        = cala_account_set.obs_chart_of_accounts.id
  member_account_set_id = cala_account_set.obs_assets.id
}

# ASSETS: Members
resource "random_uuid" "collateral_deposits_omnibus" {}
resource "cala_account" "collateral_deposits_omnibus" {
  id                  = random_uuid.collateral_deposits_omnibus.result
  name                = "Omnibus account for BTC collateral"
  code                = "BANK.COLLATERAL.OMNIBUS"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account" "collateral_deposits_omnibus_in_obs_assets" {
  account_set_id    = cala_account_set.obs_assets.id
  member_account_id = cala_account.collateral_deposits_omnibus.id
}
resource "cala_account_set_member_account" "collateral_deposits_omnibus_in_obs_trial_balance" {
  account_set_id    = cala_account_set.obs_trial_balance.id
  member_account_id = cala_account.collateral_deposits_omnibus.id
}

# LIABILITIES
resource "random_uuid" "obs_liabilities" {}
resource "cala_account_set" "obs_liabilities" {
  id                  = random_uuid.obs_liabilities.result
  journal_id          = cala_journal.journal.id
  name                = "Liabilities (Off-Balance-Sheet)"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "obs_liabilities" {
  account_set_id        = cala_account_set.obs_chart_of_accounts.id
  member_account_set_id = cala_account_set.obs_liabilities.id
}

# LIABILITIES: Members
resource "cala_account_set" "customer_collateral_control" {
  id                  = "00000000-0000-0000-0000-210000000001"
  journal_id          = cala_journal.journal.id
  name                = "Customer Collateral Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "customer_collateral_control_in_obs_liabilities" {
  account_set_id        = cala_account_set.obs_liabilities.id
  member_account_set_id = cala_account_set.customer_collateral_control.id
}
resource "cala_account_set_member_account_set" "customer_collateral_control_in_obs_trial_balance" {
  account_set_id        = cala_account_set.obs_trial_balance.id
  member_account_set_id = cala_account_set.customer_collateral_control.id
}

resource "cala_account_set" "loans_collateral_control" {
  id                  = "00000000-0000-0000-0000-210000000002"
  journal_id          = cala_journal.journal.id
  name                = "Loans Collateral Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "loans_collateral_control_in_obs_liabilities" {
  account_set_id        = cala_account_set.obs_liabilities.id
  member_account_set_id = cala_account_set.loans_collateral_control.id
}
resource "cala_account_set_member_account_set" "loans_collateral_control_in_obs_trial_balance" {
  account_set_id        = cala_account_set.obs_trial_balance.id
  member_account_set_id = cala_account_set.loans_collateral_control.id
}
