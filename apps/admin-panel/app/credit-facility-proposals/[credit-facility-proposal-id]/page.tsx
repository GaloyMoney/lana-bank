"use client"

import { use } from "react"
import { gql } from "@apollo/client"

import { useGetCreditFacilityProposalHistoryQuery } from "@/lib/graphql/generated"
import { CreditFacilityHistory } from "@/app/credit-facilities/[credit-facility-id]/history"

interface CreditFacilityProposalDetailsPageProps {
  params: Promise<{
    "credit-facility-proposal-id": string
  }>
}

gql`
  fragment CreditFacilityProposalHistoryFragment on CreditFacilityProposal {
    id
    creditFacilityProposalId
    history {
      ... on CreditFacilityIncrementalPayment {
        cents
        recordedAt
        txId
        effective
      }
      ... on CreditFacilityCollateralUpdated {
        satoshis
        recordedAt
        action
        txId
        effective
      }
      ... on CreditFacilityApproved {
        cents
        recordedAt
        txId
        effective
      }
      ... on CreditFacilityCollateralizationUpdated {
        state
        collateral
        outstandingInterest
        outstandingDisbursal
        recordedAt
        price
        effective
      }
      ... on CreditFacilityDisbursalExecuted {
        cents
        recordedAt
        txId
        effective
      }
      ... on CreditFacilityInterestAccrued {
        cents
        recordedAt
        txId
        days
        effective
      }
      ... on CreditFacilityLiquidationAmountReserved {
        cents
        recordedAt
        effective
        txId
      }
    }
  }

  query GetCreditFacilityProposalHistory($id: UUID!) {
    creditFacilityProposal(id: $id) {
      ...CreditFacilityProposalHistoryFragment
    }
  }
`

export default function CreditFacilityProposalDetailsPage({
  params,
}: CreditFacilityProposalDetailsPageProps) {
  const { "credit-facility-proposal-id": proposalId } = use(params)
  const { data } = useGetCreditFacilityProposalHistoryQuery({
    variables: { id: proposalId },
    fetchPolicy: "cache-and-network",
  })

  if (!data?.creditFacilityProposal) return null

  return <CreditFacilityHistory creditFacility={data.creditFacilityProposal} />
}
