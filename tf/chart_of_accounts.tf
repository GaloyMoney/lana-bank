# "Chart of Accounts" Account Set
resource "cala_account_set" "chart_of_accounts" {
  id         = "00000000-0000-0000-0000-210000000000"
  journal_id = cala_journal.journal.id
  name       = "Chart of Accounts"
}

# Assets
resource "cala_account_set" "coa_assets" {
  id                  = "00000000-0000-0000-0000-211000000000"
  journal_id          = cala_journal.journal.id
  name                = "Assets"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "coa_assets_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_assets.id
}

resource "cala_account_set_member_account_set" "coa_user_deposits_member" {
  account_set_id        = cala_account_set.coa_assets.id
  member_account_set_id = cala_account_set.user_deposits_control.id
}

resource "cala_account_set_member_account" "coa_bank_reserve_member" {
  account_set_id    = cala_account_set.coa_assets.id
  member_account_id = cala_account.bank_reserve.id
}


# Liabilities
resource "cala_account_set" "coa_liabilities" {
  id                  = "00000000-0000-0000-0000-212000000000"
  journal_id          = cala_journal.journal.id
  name                = "Liabilities"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_liabilities_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_liabilities.id
}

resource "cala_account_set_member_account_set" "coa_user_checking_member" {
  account_set_id        = cala_account_set.coa_liabilities.id
  member_account_set_id = cala_account_set.user_checking_control.id
}



# Equity
resource "cala_account_set" "coa_equity" {
  id                  = "00000000-0000-0000-0000-213000000000"
  journal_id          = cala_journal.journal.id
  name                = "Equity"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_equity_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_equity.id
}

resource "cala_account_set_member_account" "coa_bank_shareholder_equity_member" {
  account_set_id    = cala_account_set.coa_equity.id
  member_account_id = cala_account.bank_shareholder_equity.id
}

# Revenue
resource "cala_account_set" "coa_revenue" {
  id                  = "00000000-0000-0000-0000-214000000000"
  journal_id          = cala_journal.journal.id
  name                = "Revenue"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_revenue_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_revenue.id
}

resource "cala_account_set_member_account_set" "coa_interest_revenue_member" {
  account_set_id        = cala_account_set.coa_revenue.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}

# Expenses
resource "cala_account_set" "coa_expenses" {
  id                  = "00000000-0000-0000-0000-215000000000"
  journal_id          = cala_journal.journal.id
  name                = "Expenses"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_expenses_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_expenses.id
}
