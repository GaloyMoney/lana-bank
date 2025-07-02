"use client"
import { DetailsCard } from "@lana/web/components/details"
import React from "react"
import { useTranslations } from "next-intl"
import BigNumber from "bignumber.js"

import { GetCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { SignedUsdCents } from "@/types"

function calculateTotalCostInCents(
  creditFacility: NonNullable<GetCreditFacilityLayoutDetailsQuery["creditFacility"]>,
): number {
  const feeRateBN = new BigNumber(
    creditFacility.creditFacilityTerms.oneTimeFeeRate ?? 0,
  ).div(100)

  const facilityAmountCentsBN = new BigNumber(creditFacility.facilityAmount ?? 0)
  const oneTimeFeeCentsBN = facilityAmountCentsBN.multipliedBy(feeRateBN)
  const totalInterestCentsBN = new BigNumber(
    creditFacility.balance.interest.total.usdBalance ?? 0,
  )
  const totalCostCentsBN = totalInterestCentsBN.plus(oneTimeFeeCentsBN)
  return totalCostCentsBN.toNumber()
}

function FacilityCard({
  creditFacility,
}: {
  creditFacility: NonNullable<GetCreditFacilityLayoutDetailsQuery["creditFacility"]>
}) {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.FacilityCard")

  const totalCostUsd = calculateTotalCostInCents(creditFacility)
  const facilityData = [
    {
      label: t("details.facilityAmount"),
      value: <Balance amount={creditFacility.facilityAmount} currency="usd" />,
    },
    {
      label: t("details.facilityRemaining"),
      value: (
        <Balance
          amount={creditFacility.balance.facilityRemaining.usdBalance}
          currency="usd"
        />
      ),
    },
    {
      label: t("details.disbursedOutstanding"),
      value: (
        <Balance
          amount={creditFacility.balance.disbursed.outstanding.usdBalance}
          currency="usd"
        />
      ),
    },
    {
      label: t("details.disbursedOutstandingPayable"),
      value: (
        <Balance
          amount={creditFacility.balance.disbursed.outstandingPayable.usdBalance}
          currency="usd"
        />
      ),
    },
    {
      label: t("details.interestOutstanding"),
      value: (
        <Balance
          amount={creditFacility.balance.interest.outstanding.usdBalance}
          currency="usd"
        />
      ),
    },
    {
      label: t("details.totalOutstanding"),
      value: (
        <Balance amount={creditFacility.balance.outstanding.usdBalance} currency="usd" />
      ),
    },
    {
      label: t("details.totalInterest"),
      value: (
        <Balance
          amount={creditFacility.balance.interest.total.usdBalance}
          currency="usd"
        />
      ),
    },
    {
      label: t("details.totalDisbursed"),
      value: (
        <Balance
          amount={creditFacility.balance.disbursed.total.usdBalance}
          currency="usd"
        />
      ),
    },
    {
      label: t("details.totalCost"),
      value: <Balance amount={totalCostUsd as SignedUsdCents} currency="usd" />,
    },
  ]

  return (
    <DetailsCard
      className="w-full"
      title={t("title")}
      details={facilityData}
      columns={2}
    />
  )
}

export default FacilityCard
