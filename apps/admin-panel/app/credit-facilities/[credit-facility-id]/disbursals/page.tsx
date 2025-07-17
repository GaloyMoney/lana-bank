"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CreditFacilityDisbursals } from "./list"

import { useGetCreditFacilityDisbursalsQuery } from "@/lib/graphql/generated"

gql`
  fragment DisbursalOnFacilityPage on CreditFacilityDisbursal {
    id
    disbursalId
    amount
    status
    createdAt
  }

  query GetCreditFacilityDisbursals($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      id
      creditFacilityId
      disbursals {
        ...DisbursalOnFacilityPage
      }
    }
  }
`
export default function CreditFacilityDisbursalsPage({
  params,
}: {
  params: Promise<{ "credit-facility-id": string }>
}) {
  const { "credit-facility-id": publicId } = use(params)
  const { data } = useGetCreditFacilityDisbursalsQuery({
    variables: { publicId },
  })

  if (!data?.creditFacilityByPublicId) return null

  return <CreditFacilityDisbursals creditFacility={data.creditFacilityByPublicId} />
}
