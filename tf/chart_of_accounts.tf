# "Chart of Accounts" Account Set
resource "cala_account_set" "chart_of_accounts" {
  id         = "00000000-0000-0000-0000-100000000001"
  journal_id = cala_journal.journal.id
  name       = "Chart of Accounts"
}

resource "cala_account_set" "trial_balance" {
  id                  = "00000000-0000-0000-0000-100000000002"
  journal_id          = cala_journal.journal.id
  name                = "Trial Balance"
  normal_balance_type = "DEBIT"
}


# ASSETS
resource "random_uuid" "coa_assets" {}
resource "cala_account_set" "coa_assets" {
  id                  = random_uuid.coa_assets.result
  journal_id          = cala_journal.journal.id
  name                = "Assets"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "coa_assets_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_assets.id
}

# ASSETS: Members
resource "random_uuid" "user_deposits_control" {}
resource "cala_account_set" "user_deposits_control" {
  id                  = random_uuid.user_deposits_control.result
  journal_id          = cala_journal.journal.id
  name                = "Bank Deposits for User Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "coa_user_deposits_member" {
  account_set_id        = cala_account_set.coa_assets.id
  member_account_set_id = cala_account_set.user_deposits_control.id
}
resource "cala_account_set_member_account_set" "gl_user_deposits" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_deposits_control.id
}

resource "cala_account_set" "loans_receivable_control" {
  id                  = "00000000-0000-0000-0000-110000000001"
  journal_id          = cala_journal.journal.id
  name                = "Loans Receivable Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "coa_loans_receivable_member" {
  account_set_id        = cala_account_set.coa_assets.id
  member_account_set_id = cala_account_set.loans_receivable_control.id
}
resource "cala_account_set_member_account_set" "gl_loans" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.loans_receivable_control.id
}


resource "random_uuid" "bank_reserve" {}
resource "cala_account" "bank_reserve" {
  id                  = random_uuid.bank_reserve.result
  name                = "Bank Reserve from Shareholders"
  code                = "BANK.RESERVE_FROM_SHAREHOLDER"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account" "coa_bank_reserve_member" {
  account_set_id    = cala_account_set.coa_assets.id
  member_account_id = cala_account.bank_reserve.id
}
resource "cala_account_set_member_account" "gl_bank_reserve" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_reserve.id
}


# LIABILITIES
resource "random_uuid" "coa_liabilities" {}
resource "cala_account_set" "coa_liabilities" {
  id                  = random_uuid.coa_liabilities.result
  journal_id          = cala_journal.journal.id
  name                = "Liabilities"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_liabilities_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_liabilities.id
}

# LIABILITIES: Members
resource "cala_account_set" "user_checking_control" {
  id                  = "00000000-0000-0000-0000-120000000001"
  journal_id          = cala_journal.journal.id
  name                = "User Checking Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_user_checking_member" {
  account_set_id        = cala_account_set.coa_liabilities.id
  member_account_set_id = cala_account_set.user_checking_control.id
}
resource "cala_account_set_member_account_set" "gl_user_checking" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.user_checking_control.id
}


# EQUITY
resource "random_uuid" "coa_equity" {}
resource "cala_account_set" "coa_equity" {
  id                  = random_uuid.coa_equity.result
  journal_id          = cala_journal.journal.id
  name                = "Equity"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_equity_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_equity.id
}

# EQUITY: Members
resource "random_uuid" "bank_shareholder_equity" {}
resource "cala_account" "bank_shareholder_equity" {
  id                  = random_uuid.bank_shareholder_equity.result
  name                = "Bank Shareholder Equity"
  code                = "BANK.SHAREHOLDER_EQUITY"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account" "coa_bank_shareholder_equity_member" {
  account_set_id    = cala_account_set.coa_equity.id
  member_account_id = cala_account.bank_shareholder_equity.id
}
resource "cala_account_set_member_account" "gl_bank_shareholder_equity" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_shareholder_equity.id
}


# REVENUE
resource "random_uuid" "coa_revenue" {}
resource "cala_account_set" "coa_revenue" {
  id                  = random_uuid.coa_revenue.result
  journal_id          = cala_journal.journal.id
  name                = "Revenue"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_revenue_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_revenue.id
}

# REVENUE: Members
resource "cala_account_set" "interest_revenue_control" {
  id                  = "00000000-0000-0000-0000-140000000001"
  journal_id          = cala_journal.journal.id
  name                = "Interest Revenue Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_interest_revenue_member" {
  account_set_id        = cala_account_set.coa_revenue.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}
resource "cala_account_set_member_account_set" "gl_interest_revenue" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}


# EXPENSES
resource "random_uuid" "coa_expenses" {}
resource "cala_account_set" "coa_expenses" {
  id                  = random_uuid.coa_expenses.result
  journal_id          = cala_journal.journal.id
  name                = "Expenses"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "coa_expenses_member" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.coa_expenses.id
}

# EXPENSES: Members
# <None>
