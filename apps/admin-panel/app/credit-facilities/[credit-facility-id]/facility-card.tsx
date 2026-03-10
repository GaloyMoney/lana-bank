"use client"
import React from "react"
import { useTranslations } from "next-intl"
import BigNumber from "bignumber.js"

import { Landmark } from "lucide-react"

import { GetCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { SignedUsdCents } from "@/types"

function calculateTotalCostInCents(
  creditFacility: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >,
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
  creditFacility: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >
}) {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.FacilityCard")

  const totalCostUsd = calculateTotalCostInCents(creditFacility)
  const facilityData: DetailItemProps[] = [
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
    <div>
      <div className="flex items-center gap-2 px-4 py-2 border-b">
        <Landmark className="h-4 w-4 text-muted-foreground" />
        <h2 className="text-lg font-semibold">{t("title")}</h2>
      </div>
      <div className="pt-6 px-4 pb-4">
        <DetailsCard
          details={facilityData}
          className="w-full"
          columns={2}
          variant="container"
        />
      </div>
    </div>
  )
}

export default FacilityCard
