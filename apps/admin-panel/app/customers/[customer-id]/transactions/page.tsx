"use client"

import { gql } from "@apollo/client"

import { CustomerTransactionsTable } from "./list"

import { useGetCustomerTransactionsQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerTransactions($id: UUID!) {
    customer(id: $id) {
      id
      depositAccount {
        deposits {
          createdAt
          depositId
          reference
          amount
        }
        withdrawals {
          status
          reference
          withdrawalId
          createdAt
          amount
        }
      }
      transactions @client {
        ... on Deposit {
          createdAt
          depositId
          reference
          amount
        }
        ... on Withdrawal {
          status
          reference
          withdrawalId
          createdAt
          amount
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
  const { data } = useGetCustomerTransactionsQuery({
    variables: { id: params["customer-id"] },
  })
  if (!data?.customer) return null
  return <CustomerTransactionsTable transactions={data.customer.transactions} />
}
