"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"

import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facilities/status-badge"
import { CollateralizationStateLabel } from "@/app/credit-facilities/label"

import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"
import { formatCvl } from "@/lib/utils"

type CreditFacility = NonNullable<
  GetLiquidationDetailsQuery["liquidation"]
>["creditFacility"]

type LiquidationCreditFacilityCardProps = {
  creditFacility: CreditFacility
}

export const LiquidationCreditFacilityCard: React.FC<
  LiquidationCreditFacilityCardProps
> = ({ creditFacility }) => {
  const t = useTranslations("Liquidations.LiquidationDetails.CreditFacilityCard")

  const details: DetailItemProps[] = [
    {
      label: t("details.status"),
      value: <LoanAndCreditFacilityStatusBadge status={creditFacility.status} />,
    },
    {
      label: t("details.collateralizationState"),
      value: (
        <CollateralizationStateLabel state={creditFacility.collateralizationState} />
      ),
    },
    {
      label: t("details.maturesAt"),
      value: formatDate(creditFacility.maturesAt),
      displayCondition: creditFacility.maturesAt !== null,
    },
    {
      label: t("details.activatedAt"),
      value: formatDate(creditFacility.activatedAt),
    },
    {
      label: t("details.facilityAmount"),
      value: <Balance amount={creditFacility.facilityAmount} currency="usd" />,
    },
    {
      label: t("details.totalOutstanding"),
      value: (
        <Balance amount={creditFacility.balance.outstanding.usdBalance} currency="usd" />
      ),
    },
    {
      label: t("details.collateralBalance"),
      value: (
        <Balance amount={creditFacility.balance.collateral.btcBalance} currency="btc" />
      ),
    },
    {
      label: t("details.liquidationCvl"),
      value: formatCvl(creditFacility.creditFacilityTerms.liquidationCvl),
    },
    {
      label: t("details.currentCvl"),
      value: formatCvl(creditFacility.currentCvl),
    },
  ]

  return (
    <DetailsCard
      publicId={creditFacility.publicId}
      title={t("title")}
      details={details}
      columns={4}
    />
  )
}
