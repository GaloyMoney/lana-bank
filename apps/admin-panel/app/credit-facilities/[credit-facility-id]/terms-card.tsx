"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { ScrollText } from "lucide-react"

import { formatDate } from "@lana/web/utils"

import { GetCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { formatCvl } from "@/lib/utils"

type CreditFacilityTermsCardProps = {
  creditFacility: NonNullable<
    GetCreditFacilityLayoutDetailsQuery["creditFacilityByPublicId"]
  >
}

export const CreditFacilityTermsCard: React.FC<CreditFacilityTermsCardProps> = ({
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.TermsDialog")

  const effectiveRateDisplay = `${Number(creditFacility.creditFacilityTerms.effectiveAnnualRate).toFixed(2)}%`

  const disbursalPolicyLabel =
    creditFacility.creditFacilityTerms.disbursalPolicy === "SINGLE_DISBURSAL"
      ? t("details.singleDisbursal")
      : t("details.multipleDisbursal")

  const details: DetailItemProps[] = [
    {
      label: t("details.duration"),
      value: (
        <>
          {creditFacility.creditFacilityTerms.duration.units}{" "}
          <PeriodLabel period={creditFacility.creditFacilityTerms.duration.period} />
        </>
      ),
    },
    {
      label: t("details.interestRate"),
      value: `${creditFacility.creditFacilityTerms.annualRate}%`,
    },
    {
      label: t("details.targetCvl"),
      value: `${formatCvl(creditFacility.creditFacilityTerms.initialCvl)}`,
    },
    {
      label: t("details.marginCallCvl"),
      value: `${formatCvl(creditFacility.creditFacilityTerms.marginCallCvl)}`,
    },
    {
      label: t("details.liquidationCvl"),
      value: `${formatCvl(creditFacility.creditFacilityTerms.liquidationCvl)}`,
    },
    {
      label: t("details.dateCreated"),
      value: formatDate(creditFacility.activatedAt),
    },
    {
      label: t("details.structuringFeeRate"),
      value: `${creditFacility.creditFacilityTerms.oneTimeFeeRate}%`,
    },
    { label: t("details.effectiveRate"), value: effectiveRateDisplay },
    {
      label: t("details.disbursalPolicy"),
      value: disbursalPolicyLabel,
    },
  ]

  return (
    <div className="border-b">
      <div className="flex items-center gap-2 px-4 py-2 border-b">
        <ScrollText className="h-4 w-4 text-muted-foreground" />
        <h2 className="text-lg font-semibold">{t("title")}</h2>
      </div>
      <div className="pt-6 px-4 pb-4">
        <DetailsCard columns={4} variant="container" details={details} />
      </div>
    </div>
  )
}
