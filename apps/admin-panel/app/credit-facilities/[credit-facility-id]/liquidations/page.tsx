"use client"

import { gql } from "@apollo/client"
import { use } from "react"

import { CreditFacilityLiquidations } from "./list"

import { useGetCreditFacilityLiquidationsQuery } from "@/lib/graphql/generated"

gql`
  fragment LiquidationOnFacilityPage on Liquidation {
    id
    liquidationId
    expectedToReceive
    sentTotal
    receivedTotal
    createdAt
    completed
  }

  query GetCreditFacilityLiquidations($publicId: PublicId!) {
    creditFacilityByPublicId(id: $publicId) {
      id
      creditFacilityId
      liquidations {
        ...LiquidationOnFacilityPage
      }
    }
  }
`

export default function CreditFacilityLiquidationsPage({
  params,
}: {
  params: Promise<{ "credit-facility-id": string }>
}) {
  const { "credit-facility-id": publicId } = use(params)
  const { data } = useGetCreditFacilityLiquidationsQuery({
    variables: { publicId },
  })

  if (!data?.creditFacilityByPublicId) return null

  return <CreditFacilityLiquidations creditFacility={data.creditFacilityByPublicId} />
}
