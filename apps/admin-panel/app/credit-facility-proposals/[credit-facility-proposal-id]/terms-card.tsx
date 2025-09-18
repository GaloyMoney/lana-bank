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

  const effectiveRate =
    Number(creditFacilityProposal.creditFacilityTerms.annualRate) +
    Number(creditFacilityProposal.creditFacilityTerms.oneTimeFeeRate)

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
