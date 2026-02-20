"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { CustomerPendingCreditFacilitiesTable } from "./list"

import { useGetCustomerPendingCreditFacilitiesQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerPendingCreditFacilities($id: PublicId!) {
    customerByPublicId(id: $id) {
      id
      pendingCreditFacilities {
        id
        pendingCreditFacilityId
        createdAt
        collateralizationState
        facilityAmount
        status
        collateral {
          btcBalance
        }
      }
    }
  }
`

export default function CustomerPendingCreditFacilitiesPage({
  params,
}: {
  params: Promise<{ "customer-id": string }>
}) {
  const commonT = useTranslations("Common")

  const { "customer-id": customerId } = use(params)
  const { data, loading, error } = useGetCustomerPendingCreditFacilitiesQuery({
    variables: { id: customerId },
  })

  if (loading) return <div>{commonT("loading")}</div>
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.customerByPublicId) return <div>{commonT("notFound")}</div>

  return (
    <CustomerPendingCreditFacilitiesTable
      pendingCreditFacilities={data.customerByPublicId.pendingCreditFacilities}
    />
  )
}
