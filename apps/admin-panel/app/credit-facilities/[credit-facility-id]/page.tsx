"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CreditFacilityHistory } from "./history"

import { useGetCreditFacilityHistoryQuery } from "@/lib/graphql/generated"

gql`
  fragment CreditFacilityHistoryFragment on CreditFacility {
    id
    creditFacilityId
    history {
      ... on CreditFacilityIncrementalPayment {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityCollateralUpdated {
        satoshis
        recordedAt
        action
        txId
      }
      ... on CreditFacilityApproved {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityCollateralizationUpdated {
        state
        collateral
        outstandingInterest
        outstandingDisbursal
        recordedAt
        price
      }
      ... on CreditFacilityDisbursalExecuted {
        cents
        recordedAt
        txId
      }
      ... on CreditFacilityInterestAccrued {
        cents
        recordedAt
        txId
        days
      }
    }
  }

  query GetCreditFacilityHistory($id: UUID!) {
    creditFacility(id: $id) {
      ...CreditFacilityHistoryFragment
    }
  }
`

interface CreditFacilityHistoryPageProps {
  params: Promise<{
    "credit-facility-id": string
  }>
}

export default function CreditFacilityHistoryPage({
  params,
}: CreditFacilityHistoryPageProps) {
  const { "credit-facility-id": creditFacilityId } = use(params)
  const { data } = useGetCreditFacilityHistoryQuery({
    variables: { id: creditFacilityId },
    fetchPolicy: "cache-and-network",
  })

  if (!data?.creditFacility) return null

  return <CreditFacilityHistory creditFacility={data.creditFacility} />
}
