"use client"

import { gql } from "@apollo/client"

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

  query GetCreditFacilityDisbursals($id: UUID!) {
    creditFacility(id: $id) {
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
  params: { "credit-facility-id": string }
}) {
  const { data } = useGetCreditFacilityDisbursalsQuery({
    variables: { id: params["credit-facility-id"] },
  })

  if (!data?.creditFacility) return null

  return <CreditFacilityDisbursals creditFacility={data.creditFacility} />
}
