import { gql } from "@apollo/client"

import {
  GetTransactionHistoryDocument,
  GetTransactionHistoryQuery,
  GetTransactionHistoryQueryVariables,
} from "../generated"

import { executeQuery } from "."

gql`
  query GetTransactionHistory($first: Int!, $after: String) {
    me {
      customer {
        depositAccount {
          history(first: $first, after: $after) {
            pageInfo {
              hasNextPage
              endCursor
              hasPreviousPage
              startCursor
            }
            edges {
              cursor
              node {
                ... on DepositEntry {
                  recordedAt
                  deposit {
                    id
                    depositId
                    accountId
                    amount
                    createdAt
                    reference
                  }
                }
                ... on WithdrawalEntry {
                  recordedAt
                  withdrawal {
                    id
                    withdrawalId
                    accountId
                    amount
                    createdAt
                    reference
                    status
                  }
                }
                ... on CancelledWithdrawalEntry {
                  recordedAt
                  withdrawal {
                    id
                    withdrawalId
                    accountId
                    amount
                    createdAt
                    reference
                    status
                  }
                }
                ... on DisbursalEntry {
                  recordedAt
                  disbursal {
                    id
                    disbursalId
                    amount
                    createdAt
                    status
                  }
                }
                ... on PaymentEntry {
                  recordedAt
                  payment {
                    id
                    paymentAllocationId
                    amount
                    createdAt
                  }
                }
              }
            }
          }
        }
      }
    }
  }
`

export const getTransactionHistoryQuery = async () => {
  return executeQuery<GetTransactionHistoryQuery, GetTransactionHistoryQueryVariables>({
    document: GetTransactionHistoryDocument,
    variables: {
      first: 1000,
    },
  })
}
