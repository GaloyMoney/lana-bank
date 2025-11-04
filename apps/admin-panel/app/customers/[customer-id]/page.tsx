"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CustomerCreditFacilitiesTable } from "./credit-facilities/list"

import { useGetCustomerCreditFacilitiesQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerCreditFacilities($id: PublicId!) {
    customerByPublicId(id: $id) {
      id
      creditFacilities {
        id
        creditFacilityId
        publicId
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
`

export default function CustomerCreditFacilitiesPage({
  params,
}: {
  params: Promise<{ "customer-id": string }>
}) {
  const { "customer-id": customerId } = use(params)
  const { data } = useGetCustomerCreditFacilitiesQuery({
    variables: { id: customerId },
  })

  if (!data?.customerByPublicId) return null

  return (
    <CustomerCreditFacilitiesTable
      creditFacilities={data.customerByPublicId.creditFacilities}
    />
  )
}
