"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CreditFacilityHistory } from "./history"

import { useGetCreditFacilityHistoryQuery } from "@/lib/graphql/generated"

gql`
  fragment CreditFacilityHistoryFragment on CreditFacility {
    creditFacilityId
    history {
      ... on CreditFacilityIncrementalPayment {
        cents
        recordedAt
        paymentId
        effective
      }
      ... on CreditFacilityCollateralUpdated {
        satoshis
        recordedAt
        direction
        ledgerTransactionId
        effective
      }
      ... on CreditFacilityApproved {
        cents
        recordedAt
        ledgerTransactionId
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
        ledgerTransactionId
        effective
      }
      ... on CreditFacilityInterestAccrued {
        cents
        recordedAt
        ledgerTransactionId
        days
        effective
      }
      ... on CreditFacilityRepaymentAmountReceived {
        cents
        recordedAt
        ledgerTransactionId
        effective
      }
      ... on CreditFacilityCollateralSentOut {
        amount
        recordedAt
        ledgerTransactionId
        effective
      }
      ... on PendingCreditFacilityCollateralizationUpdated {
        pendingState: state
        collateral
        price
        recordedAt
        effective
      }
    }
  }

  query GetCreditFacilityHistory($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
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
  const { "credit-facility-id": publicId } = use(params)
  const { data } = useGetCreditFacilityHistoryQuery({
    variables: { publicId },
    fetchPolicy: "cache-and-network",
  })

  if (!data?.creditFacilityByPublicId) return null

  return <CreditFacilityHistory creditFacility={data.creditFacilityByPublicId} />
}
