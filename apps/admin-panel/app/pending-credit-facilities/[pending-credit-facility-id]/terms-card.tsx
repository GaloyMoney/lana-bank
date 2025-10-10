"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { DetailsCard, DetailItemProps } from "@/components/details"

import { GetPendingCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { formatCvl } from "@/lib/utils"

interface PendingCreditFacilityTermsCardProps {
  pendingCreditFacility: NonNullable<
    GetPendingCreditFacilityLayoutDetailsQuery["pendingCreditFacility"]
  >
}

export const PendingCreditFacilityTermsCard: React.FC<
  PendingCreditFacilityTermsCardProps
> = ({ pendingCreditFacility }) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.TermsDialog")
  const tCard = useTranslations("PendingCreditFacilities.PendingDetails.TermsCard")

  const effectiveRate =
    Number(pendingCreditFacility.creditFacilityTerms.annualRate) +
    Number(pendingCreditFacility.creditFacilityTerms.oneTimeFeeRate)

  const details: DetailItemProps[] = [
    {
      label: t("details.duration"),
      value: (
        <>
          {pendingCreditFacility.creditFacilityTerms.duration.units}{" "}
          <PeriodLabel
            period={pendingCreditFacility.creditFacilityTerms.duration.period}
          />
        </>
      ),
    },
    {
      label: t("details.interestRate"),
      value: `${pendingCreditFacility.creditFacilityTerms.annualRate}%`,
    },
    {
      label: t("details.targetCvl"),
      value: `${formatCvl(pendingCreditFacility.creditFacilityTerms.initialCvl)}`,
    },
    {
      label: t("details.marginCallCvl"),
      value: `${formatCvl(pendingCreditFacility.creditFacilityTerms.marginCallCvl)}`,
    },
    {
      label: t("details.liquidationCvl"),
      value: `${formatCvl(pendingCreditFacility.creditFacilityTerms.liquidationCvl)}`,
    },
    {
      label: t("details.structuringFeeRate"),
      value: `${pendingCreditFacility.creditFacilityTerms.oneTimeFeeRate}%`,
    },
    { label: t("details.effectiveRate"), value: `${effectiveRate}%` },
  ]

  return (
    <DetailsCard
      title={tCard("title")}
      className="w-[55%]"
      details={details}
      columns={3}
    />
  )
}
