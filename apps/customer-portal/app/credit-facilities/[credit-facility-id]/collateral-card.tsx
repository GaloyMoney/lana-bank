import { DetailsCard } from "@lana/web/components/details"
import React from "react"

import Balance, { CENTS_PER_USD, SATS_PER_BTC } from "@/components/balance"
import {
  GetCreditFacilityQuery,
  CreditFacilityStatus,
  DisbursalStatus,
} from "@/lib/graphql/generated"
import { priceQuery } from "@/lib/graphql/query/price"

type CreditFacilityData = NonNullable<GetCreditFacilityQuery["creditFacility"]>

async function CollateralCard({ data }: { data: CreditFacilityData }) {
  const priceData = await priceQuery()
  if (!priceData || priceData instanceof Error) return null

  const basisAmountInUsd = calculateBaseAmountInCents(data) / CENTS_PER_USD
  const initialCvlDecimal = data.creditFacilityTerms.initialCvl / 100
  const requiredCollateralInSats =
    (initialCvlDecimal * basisAmountInUsd * SATS_PER_BTC) /
    (priceData.realtimePrice.usdCentsPerBtc / CENTS_PER_USD)

  const basisAmountInCents = calculateBaseAmountInCents(data)
  const MarginCallPrice = calculatePrice({
    cvlPercentage: data.creditFacilityTerms.marginCallCvl,
    basisAmountInCents,
    collateralInSatoshis: data.balance.collateral.btcBalance,
  })
  const LiquidationCallPrice = calculatePrice({
    cvlPercentage: data.creditFacilityTerms.liquidationCvl,
    basisAmountInCents,
    collateralInSatoshis: data.balance.collateral.btcBalance,
  })

  const collateralData = [
    {
      label: "Collateral Balance (BTC)",
      value: <Balance amount={data.balance.collateral.btcBalance} currency="btc" />,
    },
    {
      label: "Margin Call Price (USD/BTC)",
      value: <Balance amount={MarginCallPrice} currency="usd" />,
    },
    {
      label: "Liquidation Price (USD/BTC)",
      value: <Balance amount={LiquidationCallPrice} currency="usd" />,
    },
    {
      label: `Collateral to reach target (${data.creditFacilityTerms.initialCvl}%)`,
      value: <Balance amount={requiredCollateralInSats} currency="btc" />,
    },
  ]

  return (
    <DetailsCard
      className="w-full"
      title="Collateral"
      details={collateralData}
      columns={2}
    />
  )
}

export default CollateralCard

const calculatePrice = ({
  cvlPercentage,
  basisAmountInCents,
  collateralInSatoshis,
}: {
  cvlPercentage: number
  basisAmountInCents: number
  collateralInSatoshis: number
}) => {
  if (collateralInSatoshis === 0) return 0
  const cvlDecimal = cvlPercentage / 100
  const basisAmountUsd = basisAmountInCents / CENTS_PER_USD
  const collateralBtc = collateralInSatoshis / SATS_PER_BTC
  const priceUsd = (cvlDecimal * basisAmountUsd) / collateralBtc
  const priceInCents = priceUsd * CENTS_PER_USD
  return priceInCents
}

export const calculateBaseAmountInCents = ({
  status,
  facilityAmount,
  disbursals,
  balance,
}: {
  status: CreditFacilityStatus
  facilityAmount: number
  disbursals: { status: DisbursalStatus }[]
  balance: { outstanding: { usdBalance: number } }
}) => {
  if (status === CreditFacilityStatus.Active) {
    const hasApprovedDisbursals = disbursals.some(
      (d) => d.status === DisbursalStatus.Approved,
    )
    return hasApprovedDisbursals ? balance.outstanding.usdBalance : facilityAmount
  }

  return 0
}
