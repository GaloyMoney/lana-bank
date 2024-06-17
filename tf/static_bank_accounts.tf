resource "random_uuid" "bank_shareholder_equity" {}
resource "cala_account" "bank_shareholder_equity" {
  id   = random_uuid.bank_shareholder_equity.result
  name = "Bank Shareholder Equity"
  code = "BANK.EQUITY"
  # normal_balance_type = "credit"
}

# resource "random_uuid" "bfx_shareholder_integration" {}
# resource "cala_bfx_integration" "shareholder" {
#   integration_id = random_uuid.bfx_shareholder_integration.result
#   name           = "Shareholder Equity"
#   key            = ""
#   secret         = ""
# }

# resource "random_uuid" "btc_bank_reserve_from_shareholders" {}
# resource "cala_bfx_btc_account" "btc_bank_reserve_from_shareholders" {
#   integration_id    = random_uuid.bfx_shareholder_integration.result
#   id                = random_uuid.btc_bank_reserve_from_shareholders.result
#   name              = "BTC Bank Reserve from Shareholders"
#   code              = "BANK.BTC_RESERVE_FROM_SHAREHOLDER"
#   credit_account_id = random_uuid.bank_shareholder_equity.result
# }

# resource "random_uuid" "usdt_bank_reserve_from_shareholders" {}
# resource "cala_bfx_usdt_account" "usdt_bank_reserve_from_shareholders" {
#   integration_id    = random_uuid.bfx_shareholder_integration.result
#   id                = random_uuid.usdt_bank_reserve_from_shareholders.result
#   name              = "USDT Bank Reserve from Shareholders"
#   code              = "BANK.USDT_RESERVE_FROM_SHAREHOLDER"
#   credit_account_id = random_uuid.bank_shareholder_equity.result
# }
