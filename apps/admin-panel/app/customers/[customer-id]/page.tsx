"use client"

import { gql } from "@apollo/client"

import { Card, CardContent, CardHeader, CardTitle } from "@lana/web/ui/card"

import { CustomerTransactionsTable } from "./transactions"

import { useGetCustomerTransactionHistoryQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerTransactionHistory($id: UUID!, $first: Int!, $after: String) {
    customer(id: $id) {
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

  const customerType = data?.customer?.customerType || null

  return (
    <div className="space-y-6">
      {customerType && (
        <Card>
          <CardHeader>
            <CardTitle>Customer Information</CardTitle>
          </CardHeader>
          <CardContent>
            <div>
              <p className="text-sm font-medium text-muted-foreground">Customer Type</p>
              <p className="text-lg font-semibold">{customerType}</p>
            </div>
          </CardContent>
        </Card>
      )}
      <CustomerTransactionsTable historyEntries={historyEntries} />
    </div>
  )
}
