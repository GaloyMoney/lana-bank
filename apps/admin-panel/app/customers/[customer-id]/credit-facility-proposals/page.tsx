"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { CustomerCreditFacilityProposalsTable } from "./list"

import { useGetCustomerCreditFacilityProposalsQuery } from "@/lib/graphql/generated"

gql`
  query GetCustomerCreditFacilityProposals($id: PublicId!) {
    customerByPublicId(id: $id) {
      id
      creditFacilityProposals {
        id
        creditFacilityProposalId
        createdAt
        collateralizationState
        facilityAmount
        status
        collateral {
          btcBalance
        }
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
  const t = useTranslations("Customers.CustomerDetails.creditFacilityProposals")

  const { "customer-id": customerId } = use(params)
  const { data, loading, error } = useGetCustomerCreditFacilityProposalsQuery({
    variables: { id: customerId },
  })

  if (loading) return <div>{t("loading")}</div>
  if (error) return <div className="text-destructive">{t("error")}</div>
  if (!data?.customerByPublicId) return <div>{t("notFound")}</div>

  return (
    <CustomerCreditFacilityProposalsTable
      creditFacilityProposals={data.customerByPublicId.creditFacilityProposals}
    />
  )
}