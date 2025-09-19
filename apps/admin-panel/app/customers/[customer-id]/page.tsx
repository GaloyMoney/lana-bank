"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CustomerTransactionsTable } from "./transactions"

import { useGetCustomerTransactionHistoryQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerTransactionHistory($id: PublicId!, $first: Int!, $after: String) {
    customerByPublicId(id: $id) {
      id
      customerId
      customerType
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
                  publicId
                  accountId
                  amount
                  createdAt
                  reference
                  status
                }
              }
              ... on WithdrawalEntry {
                recordedAt
                withdrawal {
                  id
                  withdrawalId
                  publicId
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
                  publicId
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
                  publicId
                  amount
                  createdAt
                  status
                }
              }
              ... on PaymentEntry {
                recordedAt
                payment {
                  id
                  obligationInstallmentId
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
`

export default function CustomerTransactionsPage({
  params,
}: {
  params: Promise<{ "customer-id": string }>
}) {
  const { "customer-id": customerId } = use(params)
  const { data, error } = useGetCustomerTransactionHistoryQuery({
    variables: {
      id: customerId,
      first: 100,
      after: null,
    },
  })
  if (error) return <div>{error.message}</div>

  const historyEntries =
    data?.customerByPublicId?.depositAccount?.history.edges.map((edge) => edge.node) || []

  return (
    <div className="space-y-6">
      <CustomerTransactionsTable historyEntries={historyEntries} />
    </div>
  )
}
