import { gql } from "@apollo/client"

gql`
  query GetTrialBalance {
    trialBalance {
      name
      balance {
        ...balancesByCurrency
      }
      memberBalances {
        ... on AccountBalance {
          name
          balance {
            ...balancesByCurrency
          }
        }
        ... on AccountSetBalance {
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
    settled {
      net
      debit
      credit
    }
    pending {
      net
      debit
      credit
    }
    encumbrance {
      net
      debit
      credit
    }
  }

  fragment usdBalances on LayeredUsdAccountBalances {
    settled {
      net
      debit
      credit
    }
    pending {
      net
      debit
      credit
    }
    encumbrance {
      net
      debit
      credit
    }
  }
`
