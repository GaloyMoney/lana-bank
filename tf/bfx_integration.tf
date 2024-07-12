variable "bitfinex_key" {
  sensitive = true
  type      = string
  default   = ""
}

variable "bitfinex_secret" {
  sensitive = true
  type      = string
  default   = ""
}

resource "cala_bitfinex_integration" "bank_deposit" {
  id         = "00000000-0000-0000-0000-200000000000"
  name       = "Bank Deposit Bitfinex Integration"
  journal_id = cala_journal.journal.id
  key        = var.bitfinex_key
  secret     = var.bitfinex_secret
  depends_on = [cala_bitfinex_integration.off_balance_sheet]
}
resource "cala_account_set_member_account" "gl_bank_deposits" {
  account_set_id    = cala_account_set.user_deposits_control.id
  member_account_id = cala_bitfinex_integration.bank_deposit.omnibus_account_id
}


resource "cala_bitfinex_integration" "off_balance_sheet" {
  id         = "10000000-0000-0000-0000-200000000000"
  name       = "Off-Balance-Sheet Bitfinex Integration"
  journal_id = cala_journal.journal.id
  key        = var.bitfinex_key
  secret     = var.bitfinex_secret
}
resource "cala_account_set_member_account" "gl_bank_collateral_deposits" {
  account_set_id    = cala_account_set.user_collateral_deposits_control.id
  member_account_id = cala_bitfinex_integration.off_balance_sheet.omnibus_account_id
}
