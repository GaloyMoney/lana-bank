"use client"

import { use } from "react"
import { gql } from "@apollo/client"

import { useTranslations } from "next-intl"

import { LiquidationDetailsCard } from "./details"
import { LiquidationCreditFacilityCard } from "./credit-facility-card"
import { LiquidationCollateralSentTable } from "./collateral-sent-table"
import { LiquidationProceedsReceivedTable } from "./payment-received-table"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useGetLiquidationDetailsQuery } from "@/lib/graphql/generated"

gql`
  fragment LiquidationCollateralSentFragment on LiquidationCollateralSent {
    amount
    ledgerTxId
  }

  fragment LiquidationProceedsReceivedFragment on LiquidationProceedsReceived {
    amount
    ledgerTxId
  }

  fragment LiquidationDetails on Liquidation {
    id
    liquidationId
    creditFacilityId
    expectedToReceive
    sentTotal
    amountReceived
    createdAt
    completed
    creditFacility {
      id
      creditFacilityId
      publicId
      status
      collateralizationState
      facilityAmount
      activatedAt
      maturesAt
      currentCvl {
        __typename
        ... on FiniteCVLPct {
          value
        }
        ... on InfiniteCVLPct {
          isInfinite
        }
      }
      creditFacilityTerms {
        liquidationCvl {
          __typename
          ... on FiniteCVLPct {
            value
          }
          ... on InfiniteCVLPct {
            isInfinite
          }
        }
      }
      balance {
        outstanding {
          usdBalance
        }
        collateral {
          btcBalance
        }
      }
      customer {
        customerId
        publicId
        customerType
        email
      }
    }
    sentCollateral {
      ...LiquidationCollateralSentFragment
    }
    receivedProceeds {
      ...LiquidationProceedsReceivedFragment
    }
  }

  query GetLiquidationDetails($liquidationId: UUID!) {
    liquidation(id: $liquidationId) {
      ...LiquidationDetails
    }
  }
`

function LiquidationPage({
  params,
}: {
  params: Promise<{
    "liquidation-id": string
  }>
}) {
  const { "liquidation-id": liquidationId } = use(params)
  const { data, loading, error } = useGetLiquidationDetailsQuery({
    variables: { liquidationId },
  })
  const commonT = useTranslations("Common")

  if (loading) {
    return <DetailsPageSkeleton tabs={0} detailItems={6} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.liquidation) return <div>{commonT("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <LiquidationDetailsCard liquidation={data.liquidation} />
      <LiquidationCreditFacilityCard creditFacility={data.liquidation.creditFacility} />
      <div className="flex flex-col md:flex-row gap-2 items-start">
        <LiquidationCollateralSentTable
          collateralSent={data.liquidation.sentCollateral}
        />
        <LiquidationProceedsReceivedTable
          paymentsReceived={data.liquidation.receivedProceeds}
        />
      </div>
    </main>
  )
}

export default LiquidationPage
