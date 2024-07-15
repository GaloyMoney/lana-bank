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

resource "cala_bitfinex_integration" "bank_deposits" {
  id         = "00000000-0000-0000-0000-200000000000"
  name       = "Bank Deposit Bitfinex Integration"
  journal_id = cala_journal.journal.id
  key        = var.bitfinex_key
  secret     = var.bitfinex_secret
  depends_on = [cala_bitfinex_integration.obs_collateral_deposits]
}
resource "cala_account_set_member_account" "bank_deposits_in_bank_deposits_control" {
  account_set_id    = cala_account_set.bank_deposits_control.id
  member_account_id = cala_bitfinex_integration.bank_deposits.omnibus_account_id
}


resource "cala_bitfinex_integration" "obs_collateral_deposits" {
  id         = "10000000-0000-0000-0000-200000000000"
  name       = "Off-Balance-Sheet Collateral Deposit Bitfinex Integration"
  journal_id = cala_journal.journal.id
  key        = var.bitfinex_key
  secret     = var.bitfinex_secret
}
resource "cala_account_set_member_account" "obs_collateral_deposits_in_collateral_deposits_control" {
  account_set_id    = cala_account_set.collateral_deposits_control.id
  member_account_id = cala_bitfinex_integration.obs_collateral_deposits.omnibus_account_id
}
