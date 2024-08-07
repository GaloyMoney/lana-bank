resource "cala_account_set" "net_income" {
  id                  = "00000000-0000-0000-0000-100000000004"
  journal_id          = cala_journal.journal.id
  name                = "Net Income"
  normal_balance_type = "CREDIT"
}


# REVENUE
resource "cala_account_set_member_account_set" "revenue_in_net_income" {
  account_set_id        = cala_account_set.net_income.id
  member_account_set_id = cala_account_set.revenue.id
}


# EXPENSES
resource "cala_account_set_member_account_set" "expenses_in_net_income" {
  account_set_id        = cala_account_set.net_income.id
  member_account_set_id = cala_account_set.expenses.id
}
