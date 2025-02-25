"use client"

import { gql } from "@apollo/client"

import { CustomerTransactionsTable } from "./transactions"

import { useGetCustomerTransactionHistoryQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerTransactionHistory($id: UUID!, $first: Int!, $after: String) {
    customer(id: $id) {
      id
      customerId
      depositAccount {
        depositAccountId
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
                  index
                  amount
                  createdAt
                  status
                }
              }
              ... on PaymentEntry {
                recordedAt
                payment {
                  id
                  paymentId
                  interestAmount
                  disbursalAmount
                  createdAt
                }
              }
            }
          }
        }
      }
    }
  }
`

export default function CustomerTransactionsPage({
  params,
}: {
  params: { "customer-id": string }
}) {
  const { data, error } = useGetCustomerTransactionHistoryQuery({
    variables: {
      id: params["customer-id"],
      first: 100,
      after: null,
    },
  })
  if (error) return <div>{error.message}</div>

  const historyEntries =
    data?.customer?.depositAccount?.history.edges.map((edge) => edge.node) || []

  return <CustomerTransactionsTable historyEntries={historyEntries} />
}
