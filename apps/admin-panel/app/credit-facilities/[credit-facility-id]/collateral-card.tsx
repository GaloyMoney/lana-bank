import React from "react"
import { useTranslations } from "next-intl"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"
import {
  CreditFacilityStatus,
  CvlPctDataFragment,
  DisbursalStatus,
  GetCreditFacilityLayoutDetailsQuery,
  useGetRealtimePriceUpdatesQuery,
} from "@/lib/graphql/generated"
import { CENTS_PER_USD, formatCvl, getCvlValue, SATS_PER_BTC } from "@/lib/utils"
import { Satoshis, UsdCents } from "@/types"

type CreditFacilityOverviewProps = {
  creditFacility: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >
}

const getCvlStatus = (
  currentCvl: CvlPctDataFragment,
  initialCvl: CvlPctDataFragment,
  marginCallCvl: CvlPctDataFragment,
  liquidationCvl: CvlPctDataFragment,
  t: (key: string) => string,
) => {
  const currentVal = getCvlValue(currentCvl)
  const initialVal = getCvlValue(initialCvl)
  const marginVal = getCvlValue(marginCallCvl)
  const liquidationVal = getCvlValue(liquidationCvl)
  if (currentVal >= initialVal) return { label: null, color: null }
  if (currentVal >= marginVal)
    return { label: t("status.moderate"), color: "text-warning" }
  if (currentVal >= liquidationVal)
    return { label: t("status.high"), color: "text-warning" }
  return { label: t("status.critical"), color: "text-destructive" }
}

const CvlStatusText: React.FC<{
  currentCvl: CvlPctDataFragment
  initialCvl: CvlPctDataFragment
  marginCallCvl: CvlPctDataFragment
  liquidationCvl: CvlPctDataFragment
  t: (key: string) => string
}> = ({ currentCvl, initialCvl, marginCallCvl, liquidationCvl, t }) => {
  const { label, color } = getCvlStatus(
    currentCvl,
    initialCvl,
    marginCallCvl,
    liquidationCvl,
    t,
  )
  if (label && color) return <span className={`font-medium ${color}`}>{label}</span>
  return <></>
}

export const CreditFacilityCollateral: React.FC<CreditFacilityOverviewProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.CollateralCard")

  const basisAmountInCents = calculateBaseAmountInCents(creditFacility)
  const MarginCallPrice = calculatePrice({
    cvlPercentage: getCvlValue(creditFacility.creditFacilityTerms.marginCallCvl),
    basisAmountInCents,
    collateralInSatoshis: creditFacility.balance.collateral.btcBalance,
  })
  const LiquidationCallPrice = calculatePrice({
    cvlPercentage: getCvlValue(creditFacility.creditFacilityTerms.liquidationCvl),
    basisAmountInCents,
    collateralInSatoshis: creditFacility.balance.collateral.btcBalance,
  })

  const { data: priceInfo } = useGetRealtimePriceUpdatesQuery({
    fetchPolicy: "cache-only",
  })

  const collateralInUsd = priceInfo
    ? (creditFacility.balance.collateral.btcBalance / SATS_PER_BTC) *
      (priceInfo.realtimePrice.usdCentsPerBtc / CENTS_PER_USD)
    : 0

  const collateralDependentDetails: DetailItemProps[] = [
    {
      label: t("details.collateralBalance"),
      value: (
        <Balance amount={creditFacility.balance.collateral.btcBalance} currency="btc" />
      ),
    },
    {
      label: t("details.currentPrice"),
      value: priceInfo && (
        <Balance amount={priceInfo.realtimePrice.usdCentsPerBtc} currency="usd" />
      ),
    },
    {
      label: t("details.collateralValue"),
      value: priceInfo && (
        <Balance amount={(collateralInUsd * CENTS_PER_USD) as UsdCents} currency="usd" />
      ),
    },
    {
      label: t("details.marginCallPrice", {
        percentage: formatCvl(creditFacility.creditFacilityTerms.marginCallCvl),
      }),
      value: <Balance amount={MarginCallPrice as UsdCents} currency="usd" />,
      displayCondition: getCvlValue(creditFacility.currentCvl) != Infinity,
    },
    {
      label: t("details.liquidationPrice", {
        percentage: formatCvl(creditFacility.creditFacilityTerms.liquidationCvl),
      }),
      value: <Balance amount={LiquidationCallPrice as UsdCents} currency="usd" />,
      displayCondition: getCvlValue(creditFacility.currentCvl) != Infinity,
    },
    {
      label: t("details.collateralToReachTarget", {
        percentage: formatCvl(creditFacility.creditFacilityTerms.initialCvl),
      }),
      value: (
        <Balance
          amount={(creditFacility.collateralToMatchInitialCvl ?? 0) as Satoshis}
          currency="btc"
        />
      ),
      valueTestId: "collateral-to-reach-target",
    },
    {
      label: t("details.currentCvl"),
      value: (
        <div className="flex items-center gap-2">
          <span>{formatCvl(creditFacility.currentCvl)}</span>
          {creditFacility.status === CreditFacilityStatus.Active && (
            <CvlStatusText
              currentCvl={creditFacility.currentCvl}
              initialCvl={creditFacility.creditFacilityTerms.initialCvl}
              marginCallCvl={creditFacility.creditFacilityTerms.marginCallCvl}
              liquidationCvl={creditFacility.creditFacilityTerms.liquidationCvl}
              t={t}
            />
          )}
        </div>
      ),
    },
  ]

  return (
    <DetailsCard
      className="w-full"
      title={t("title")}
      details={collateralDependentDetails}
      columns={2}
    />
  )
}

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
