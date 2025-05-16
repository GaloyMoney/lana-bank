"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CreditFacilityHistory } from "./history"

import {
  useGetCreditFacilityHistoryQuery,
  useGetCreditFacilityLayoutDetailsQuery,
  useGetCustomerBasicDetailsQuery,
} from "@/lib/graphql/generated"
import { removeUnderscore } from "@/lib/utils"

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
      ... on CreditFacilityOrigination {
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

  query GetCreditFacilityLayoutDetails($id: UUID!) {
    creditFacility(id: $id) {
      id
      createdAt
      maturesAt
      customer {
        customerId
      }
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
  const { data: cfData } = useGetCreditFacilityHistoryQuery({
    variables: { id: creditFacilityId },
    fetchPolicy: "cache-and-network",
  })

  const { data: layoutData } = useGetCreditFacilityLayoutDetailsQuery({
    variables: { id: creditFacilityId },
  })

  const customerId = layoutData?.creditFacility?.customer?.customerId
  const { data: customerData } = useGetCustomerBasicDetailsQuery({
    variables: { id: customerId! },
    skip: !customerId,
  })

  if (!cfData?.creditFacility || !layoutData?.creditFacility) return null

  const customerType = customerData?.customer?.customerType
  const customerTypeDisplay = customerType ? removeUnderscore(customerType) : "Unknown"

  const issuanceDate = new Date(layoutData.creditFacility.createdAt).toLocaleDateString()
  const maturityDate = layoutData.creditFacility.maturesAt
    ? new Date(layoutData.creditFacility.maturesAt).toLocaleDateString()
    : "Not set"

  return (
    <div>
      <div className="space-y-2 mb-4">
        <div>
          <span className="font-medium">Customer Type: </span>
          <span>{customerTypeDisplay}</span>
        </div>
        <div>
          <span className="font-medium">Date of Issuance: </span>
          <span>{issuanceDate}</span>
        </div>
        <div>
          <span className="font-medium">Maturity Date: </span>
          <span>{maturityDate}</span>
        </div>
      </div>
      <CreditFacilityHistory creditFacility={cfData.creditFacility} />
    </div>
  )
}
