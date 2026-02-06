"use client"

import React from "react"
import Link from "next/link"
import { useTranslations } from "next-intl"
import { ArrowRight } from "lucide-react"

import { formatDate } from "@lana/web/utils"
import { Button } from "@lana/web/ui/button"

import Balance from "@/components/balance/balance"
import { DetailsCard, DetailItemProps } from "@/components/details"

import { LoanAndCreditFacilityStatusBadge } from "@/app/credit-facilities/status-badge"
import { CollateralizationStateLabel } from "@/app/credit-facilities/label"

import { GetLiquidationDetailsQuery } from "@/lib/graphql/generated"
import { formatCvl } from "@/lib/utils"

type CreditFacility = NonNullable<
  GetLiquidationDetailsQuery["liquidation"]
>["collateral"]["creditFacility"]

type LiquidationCreditFacilityCardProps = {
  creditFacility: CreditFacility
}

export const LiquidationCreditFacilityCard: React.FC<
  LiquidationCreditFacilityCardProps
> = ({ creditFacility }) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails")
  const buttonsT = useTranslations("Liquidations.LiquidationDetails.DetailsCard.buttons")

  const details: DetailItemProps[] = [
    {
      label: t("DetailsCard.details.status"),
      value: <LoanAndCreditFacilityStatusBadge status={creditFacility.status} />,
    },
    {
      label: t("DetailsCard.details.collateralizationState"),
      value: (
        <CollateralizationStateLabel state={creditFacility.collateralizationState} />
      ),
    },
    {
      label: t("DetailsCard.details.dateOfIssuance"),
      value: formatDate(creditFacility.activatedAt),
    },
    {
      label: t("DetailsCard.details.maturityDate"),
      value: formatDate(creditFacility.maturesAt),
      displayCondition: creditFacility.maturesAt !== null,
    },
    {
      label: t("FacilityCard.details.facilityAmount"),
      value: <Balance amount={creditFacility.facilityAmount} currency="usd" />,
    },
    {
      label: t("FacilityCard.details.totalOutstanding"),
      value: (
        <Balance amount={creditFacility.balance.outstanding.usdBalance} currency="usd" />
      ),
    },
    {
      label: t("CollateralCard.details.collateralBalance"),
      value: (
        <Balance amount={creditFacility.balance.collateral.btcBalance} currency="btc" />
      ),
    },
    {
      label: t("TermsDialog.details.liquidationCvl"),
      value: formatCvl(creditFacility.creditFacilityTerms.liquidationCvl),
    },
    {
      label: t("CollateralCard.details.currentCvl"),
      value: formatCvl(creditFacility.currentCvl),
    },
  ]

  const footerContent = (
    <Button variant="outline" asChild>
      <Link href={`/credit-facilities/${creditFacility.publicId}`}>
        {buttonsT("viewMoreDetails")}
        <ArrowRight className="h-4 w-4 ml-2" />
      </Link>
    </Button>
  )

  return (
    <DetailsCard
      publicId={creditFacility.publicId}
      title={t("DetailsCard.title")}
      details={details}
      columns={4}
      footerContent={footerContent}
    />
  )
}
