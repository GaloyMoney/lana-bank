import { gql } from "@apollo/client"

gql`
  query GetTrialBalance {
    trialBalance {
      name
      balance {
        ...balancesByCurrency
      }
      subAccounts {
        ... on AccountWithBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
        ... on AccountSetWithBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
      }
    }
  }

  fragment balancesByCurrency on AccountBalancesByCurrency {
    btc: btc {
      ...btcBalances
    }
    usd: usd {
      ...usdBalances
    }
    usdt: usdt {
      ...usdBalances
    }
  }

  fragment btcBalances on LayeredBtcAccountBalances {
    all {
      netDebit
      debit
      credit
    }
    settled {
      netDebit
      debit
      credit
    }
    pending {
      netDebit
      debit
      credit
    }
    encumbrance {
      netDebit
      debit
      credit
    }
  }

  fragment usdBalances on LayeredUsdAccountBalances {
    all {
      netDebit
      debit
      credit
    }
    settled {
      netDebit
      debit
      credit
    }
    pending {
      netDebit
      debit
      credit
    }
    encumbrance {
      netDebit
      debit
      credit
    }
  }
`
