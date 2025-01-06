"use client"

import { gql } from "@apollo/client"

import { CustomerAccountBalances } from "./balances"
import { KycStatus } from "./kyc-status"

import { useGetCustomerOverviewQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerOverview($id: UUID!) {
    customer(id: $id) {
      id
      customerId
      depositAccount {
        balance {
          settled
          pending
        }
      }
    }
  }
`

export default function CustomerPage({ params }: { params: { "customer-id": string } }) {
  const { data } = useGetCustomerOverviewQuery({
    variables: { id: params["customer-id"] },
  })

  if (!data?.customer) return null

  return (
    <div className="flex flex-col md:flex-row w-full gap-2">
      <CustomerAccountBalances balance={data.customer.depositAccount.balance} />
      <KycStatus customerId={params["customer-id"]} />
    </div>
  )
}
