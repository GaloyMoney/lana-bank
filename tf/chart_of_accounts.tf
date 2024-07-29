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

resource "cala_account_set" "balance_sheet" {
  id                  = "00000000-0000-0000-0000-100000000003"
  journal_id          = cala_journal.journal.id
  name                = "Balance Sheet"
  normal_balance_type = "DEBIT"
}

resource "cala_account_set" "net_income" {
  id                  = "00000000-0000-0000-0000-100000000004"
  journal_id          = cala_journal.journal.id
  name                = "Net Income"
  normal_balance_type = "CREDIT"
}

# ASSETS
resource "random_uuid" "assets" {}
resource "cala_account_set" "assets" {
  id                  = random_uuid.assets.result
  journal_id          = cala_journal.journal.id
  name                = "Assets"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "assets" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.assets.id
}
resource "cala_account_set_member_account_set" "assets_in_balance_sheet" {
  account_set_id        = cala_account_set.balance_sheet.id
  member_account_set_id = cala_account_set.assets.id
}

# ASSETS: Members
resource "random_uuid" "bank_deposits_control" {}
resource "cala_account_set" "bank_deposits_control" {
  id                  = random_uuid.bank_deposits_control.result
  journal_id          = cala_journal.journal.id
  name                = "Bank Deposits from Users Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "bank_deposits_control_in_assets" {
  account_set_id        = cala_account_set.assets.id
  member_account_set_id = cala_account_set.bank_deposits_control.id
}
resource "cala_account_set_member_account_set" "bank_deposits_control_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.bank_deposits_control.id
}

resource "cala_account_set" "loans_receivable_control" {
  id                  = "00000000-0000-0000-0000-110000000001"
  journal_id          = cala_journal.journal.id
  name                = "Loans Receivable Control Account"
  normal_balance_type = "DEBIT"
}
resource "cala_account_set_member_account_set" "loans_receivable_control_in_assets" {
  account_set_id        = cala_account_set.assets.id
  member_account_set_id = cala_account_set.loans_receivable_control.id
}
resource "cala_account_set_member_account_set" "loans_receivable_control_in_trial_balance" {
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
resource "cala_account_set_member_account" "bank_reserve_in_assets" {
  account_set_id    = cala_account_set.assets.id
  member_account_id = cala_account.bank_reserve.id
}
resource "cala_account_set_member_account" "bank_reserve_in_trial_balance" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_reserve.id
}


# LIABILITIES
resource "random_uuid" "liabilities" {}
resource "cala_account_set" "liabilities" {
  id                  = random_uuid.liabilities.result
  journal_id          = cala_journal.journal.id
  name                = "Liabilities"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "liabilities" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.liabilities.id
}
resource "cala_account_set_member_account_set" "liabilities_in_balance_sheet" {
  account_set_id        = cala_account_set.balance_sheet.id
  member_account_set_id = cala_account_set.liabilities.id
}

# LIABILITIES: Members
resource "cala_account_set" "customer_checking_control" {
  id                  = "00000000-0000-0000-0000-120000000001"
  journal_id          = cala_journal.journal.id
  name                = "Customer Checking Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "customer_checking_in_liabilities" {
  account_set_id        = cala_account_set.liabilities.id
  member_account_set_id = cala_account_set.customer_checking_control.id
}
resource "cala_account_set_member_account_set" "customer_checking_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.customer_checking_control.id
}


# EQUITY
resource "random_uuid" "equity" {}
resource "cala_account_set" "equity" {
  id                  = random_uuid.equity.result
  journal_id          = cala_journal.journal.id
  name                = "Equity"
  normal_balance_type = "CREDIT"
}
resource "random_uuid" "equity_for_chart_of_accounts" {}
resource "cala_account_set" "equity_for_chart_of_accounts" {
  id                  = random_uuid.equity_for_chart_of_accounts.result
  journal_id          = cala_journal.journal.id
  name                = "Equity"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "equity_in_chart_of_accounts" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.equity_for_chart_of_accounts.id
}
resource "cala_account_set_member_account_set" "equity_in_balance_sheet" {
  account_set_id        = cala_account_set.balance_sheet.id
  member_account_set_id = cala_account_set.equity.id
}
resource "cala_account_set_member_account_set" "net_income_in_equity" {
  account_set_id        = cala_account_set.equity.id
  member_account_set_id = cala_account_set.net_income.id
}


# EQUITY: Members
resource "random_uuid" "bank_shareholder_equity" {}
resource "cala_account" "bank_shareholder_equity" {
  id                  = random_uuid.bank_shareholder_equity.result
  name                = "Bank Shareholder Equity"
  code                = "BANK.SHAREHOLDER_EQUITY"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account" "bank_shareholder_equity_in_equity" {
  account_set_id    = cala_account_set.equity.id
  member_account_id = cala_account.bank_shareholder_equity.id
}
resource "cala_account_set_member_account" "bank_shareholder_equity_in_equity_coa" {
  account_set_id    = cala_account_set.equity_for_chart_of_accounts.id
  member_account_id = cala_account.bank_shareholder_equity.id
}
resource "cala_account_set_member_account" "bank_shareholder_equity_in_trial_balance" {
  account_set_id    = cala_account_set.trial_balance.id
  member_account_id = cala_account.bank_shareholder_equity.id
}


# REVENUE
resource "random_uuid" "revenue" {}
resource "cala_account_set" "revenue" {
  id                  = random_uuid.revenue.result
  journal_id          = cala_journal.journal.id
  name                = "Revenue"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "revenue" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.revenue.id
}
resource "cala_account_set_member_account_set" "revenue_in_net_income" {
  account_set_id        = cala_account_set.net_income.id
  member_account_set_id = cala_account_set.revenue.id
}

# REVENUE: Members
resource "cala_account_set" "interest_revenue_control" {
  id                  = "00000000-0000-0000-0000-140000000001"
  journal_id          = cala_journal.journal.id
  name                = "Interest Revenue Control Account"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "interest_revenue_control_in_revenue" {
  account_set_id        = cala_account_set.revenue.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}
resource "cala_account_set_member_account_set" "interest_revenue_in_trial_balance" {
  account_set_id        = cala_account_set.trial_balance.id
  member_account_set_id = cala_account_set.interest_revenue_control.id
}


# EXPENSES
resource "random_uuid" "expenses" {}
resource "cala_account_set" "expenses" {
  id                  = random_uuid.expenses.result
  journal_id          = cala_journal.journal.id
  name                = "Expenses"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "expenses" {
  account_set_id        = cala_account_set.chart_of_accounts.id
  member_account_set_id = cala_account_set.expenses.id
}
resource "cala_account_set_member_account_set" "expenses_in_net_income" {
  account_set_id        = cala_account_set.net_income.id
  member_account_set_id = cala_account_set.expenses.id
}

# EXPENSES: Members
# <None>
