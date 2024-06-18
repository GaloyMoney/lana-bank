variable "bitfinex_key" {
  sensitive = true
  type      = string
}

variable "bitfinex_secret" {
  sensitive = true
  type      = string
}

resource "cala_bitfinex_integration" "off_balance_sheet" {
  id         = "00000000-0000-0000-0000-200000000001"
  name       = "Off-Balance-Sheet Bitfinex Integration"
  journal_id = cala_journal.journal.id
  key        = var.bitfinex_key
  secret     = var.bitfinex_secret
}

resource "cala_bitfinex_integration" "bank_deposit" {
  id         = "00000000-0000-0000-0000-200000000002"
  name       = "Bank Deposit Bitfinex Integration"
  journal_id = cala_journal.journal.id
  key        = var.bitfinex_key
  secret     = var.bitfinex_secret
}
