"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { CustomerCreditFacilityProposalsTable } from "./list"

import { NotFound } from "@/components/not-found"


import { useGetCustomerCreditFacilityProposalsQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerCreditFacilityProposals($id: PublicId!) {
    customerByPublicId(id: $id) {
      id
      creditFacilityProposals {
        id
        creditFacilityProposalId
        createdAt
        facilityAmount
        status
        customer {
          customerId
          email
        }
      }
    }
  }
`

export default function CustomerCreditFacilityProposalsPage({
  params,
}: {
  params: Promise<{ "customer-id": string }>
}) {
  const commonT = useTranslations("Common")

  const { "customer-id": customerId } = use(params)
  const { data, loading, error } = useGetCustomerCreditFacilityProposalsQuery({
    variables: { id: customerId },
  })

  if (loading) return <div>{commonT("loading")}</div>
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.customerByPublicId) return <NotFound />

  return (
    <CustomerCreditFacilityProposalsTable
      creditFacilityProposals={data.customerByPublicId.creditFacilityProposals}
    />
  )
}
