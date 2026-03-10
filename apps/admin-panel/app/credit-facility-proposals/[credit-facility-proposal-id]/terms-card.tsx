"use client"

import React from "react"
import { useTranslations } from "next-intl"

import { GetCreditFacilityProposalLayoutDetailsQuery } from "@/lib/graphql/generated"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { formatCvl } from "@/lib/utils"

type CreditFacilityTermsCardProps = {
  creditFacilityProposal: NonNullable<
    GetCreditFacilityProposalLayoutDetailsQuery["creditFacilityProposal"]
  >
}

export const CreditFacilityTermsCard: React.FC<CreditFacilityTermsCardProps> = ({
  creditFacilityProposal,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.TermsDialog")
  const tCard = useTranslations("CreditFacilityProposals.ProposalDetails.TermsCard")

  const effectiveRateDisplay = `${Number(creditFacilityProposal.creditFacilityTerms.effectiveAnnualRate).toFixed(2)}%`

  const disbursalPolicyLabel =
    creditFacilityProposal.creditFacilityTerms.disbursalPolicy === "SINGLE_DISBURSAL"
      ? t("details.singleDisbursal")
      : t("details.multipleDisbursal")

  const details: DetailItemProps[] = [
    {
      label: t("details.duration"),
      value: (
        <>
          {creditFacilityProposal.creditFacilityTerms.duration.units}{" "}
          <PeriodLabel
            period={creditFacilityProposal.creditFacilityTerms.duration.period}
          />
        </>
      ),
    },
    {
      label: t("details.interestRate"),
      value: `${creditFacilityProposal.creditFacilityTerms.annualRate}%`,
    },
    {
      label: t("details.targetCvl"),
      value: `${formatCvl(creditFacilityProposal.creditFacilityTerms.initialCvl)}`,
    },
    {
      label: t("details.marginCallCvl"),
      value: `${formatCvl(creditFacilityProposal.creditFacilityTerms.marginCallCvl)}`,
    },
    {
      label: t("details.liquidationCvl"),
      value: `${formatCvl(creditFacilityProposal.creditFacilityTerms.liquidationCvl)}`,
    },
    {
      label: t("details.structuringFeeRate"),
      value: `${creditFacilityProposal.creditFacilityTerms.oneTimeFeeRate}%`,
    },
    { label: t("details.effectiveRate"), value: effectiveRateDisplay },
    {
      label: t("details.disbursalPolicy"),
      value: disbursalPolicyLabel,
    },
  ]

  return (
    <div>
      <div className="flex items-center gap-2 px-4 py-2 border-b">
        <h2 className="text-lg font-semibold">{tCard("title")}</h2>
      </div>
      <div className="p-4 border-b">
        <DetailsCard
          details={details}
          className="w-full"
          columns={3}
          variant="container"
        />
      </div>
    </div>
  )
}
