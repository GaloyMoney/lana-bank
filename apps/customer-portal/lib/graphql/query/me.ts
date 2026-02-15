import { gql } from "@apollo/client"

import { MeDocument, MeQuery, MeQueryVariables } from "../generated"

import { executeQuery } from "."

gql`
  query me {
    me {
      customer {
        id
        customerId
        kycVerification
        level
        createdAt
        email
        telegramHandle
        depositAccount {
          id
          depositAccountId
          customerId
          createdAt
          balance {
            settled
            pending
          }
          deposits {
            id
            depositId
            accountId
            amount
            createdAt
            reference
          }
          withdrawals {
            id
            withdrawalId
            accountId
            amount
            createdAt
            reference
            status
          }
        }
        creditFacilities {
          id
          creditFacilityId
          collateralizationState
          status
          activatedAt
          balance {
            collateral {
              btcBalance
            }
            outstanding {
              usdBalance
            }
          }
        }
      }
    }
  }
`

export const meQuery = async () => {
  return executeQuery<MeQuery, MeQueryVariables>({
    document: MeDocument,
    variables: {},
  })
}
