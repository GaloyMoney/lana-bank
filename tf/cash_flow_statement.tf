resource "cala_account_set" "cash_flow" {
  id                  = "00000000-0000-0000-0000-100000000005"
  journal_id          = cala_journal.journal.id
  name                = "Cash Flow Statement"
  normal_balance_type = "CREDIT"
}

# CASH FLOW FROM OPERATIONS
resource "random_uuid" "from_operations" {}
resource "cala_account_set" "from_operations" {
  id                  = random_uuid.from_operations.result
  journal_id          = cala_journal.journal.id
  name                = "Cash Flow From Operations"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "from_operations" {
  account_set_id        = cala_account_set.cash_flow.id
  member_account_set_id = cala_account_set.from_operations.id
}

# CASH FLOW FROM OPERATIONS: Members
resource "cala_account_set_member_account_set" "net_income_in_operations" {
  account_set_id        = cala_account_set.from_operations.id
  member_account_set_id = cala_account_set.net_income.id
}
resource "cala_account_set_member_account_set" "loans_interest_receivable_control_in_operations" {
  account_set_id        = cala_account_set.from_operations.id
  member_account_set_id = cala_account_set.loans_interest_receivable_control.id
}

# CASH FLOW FROM INVESTING
resource "random_uuid" "from_investing" {}
resource "cala_account_set" "from_investing" {
  id                  = random_uuid.from_investing.result
  journal_id          = cala_journal.journal.id
  name                = "Cash Flow From Investing"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "from_investing" {
  account_set_id        = cala_account_set.cash_flow.id
  member_account_set_id = cala_account_set.from_investing.id
}

# CASH FLOW FROM INVESTING: Members
# <None>

# CASH FLOW FROM FINANCING
resource "random_uuid" "from_financing" {}
resource "cala_account_set" "from_financing" {
  id                  = random_uuid.from_financing.result
  journal_id          = cala_journal.journal.id
  name                = "Cash Flow From Financing"
  normal_balance_type = "CREDIT"
}
resource "cala_account_set_member_account_set" "from_financing" {
  account_set_id        = cala_account_set.cash_flow.id
  member_account_set_id = cala_account_set.from_financing.id
}

# CASH FLOW FROM FINANCING: Members
# <None>
