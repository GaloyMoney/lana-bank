"use client"

import { use } from "react"
import { gql } from "@apollo/client"

import { useTranslations } from "next-intl"

import { LiquidationDetailsCard } from "./details"
import { LiquidationCollateralSentTable } from "./collateral-sent-table"
import { LiquidationPaymentReceivedTable } from "./payment-received-table"

import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useGetLiquidationDetailsQuery } from "@/lib/graphql/generated"

gql`
  fragment LiquidationCollateralSentFragment on LiquidationCollateralSent {
    amount
    ledgerTxId
  }

  fragment LiquidationPaymentReceivedFragment on LiquidationPaymentReceived {
    amount
    ledgerTxId
  }

  query GetLiquidationDetails($liquidationId: UUID!) {
    liquidation(id: $liquidationId) {
      id
      liquidationId
      expectedToReceive
      sentTotal
      receivedTotal
      createdAt
      completed
      sentCollateral {
        ...LiquidationCollateralSentFragment
      }
      receivedPayment {
        ...LiquidationPaymentReceivedFragment
      }
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
    return <DetailsPageSkeleton tabs={0} detailItems={5} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.liquidation) return <div>{commonT("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto space-y-4">
      <LiquidationDetailsCard liquidation={data.liquidation} />
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <LiquidationCollateralSentTable
          collateralSent={data.liquidation.sentCollateral}
        />
        <LiquidationPaymentReceivedTable
          paymentsReceived={data.liquidation.receivedPayment}
        />
      </div>
    </main>
  )
}

export default LiquidationPage
