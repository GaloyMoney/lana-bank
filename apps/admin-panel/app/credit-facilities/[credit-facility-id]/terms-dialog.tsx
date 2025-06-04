"use client"

import React from "react"
import { useTranslations } from "next-intl"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@lana/web/ui/dialog"

import { formatDate } from "@lana/web/utils"

import { GetCreditFacilityLayoutDetailsQuery } from "@/lib/graphql/generated"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { DetailsCard, DetailItemProps } from "@/components/details"

type CreditFacilityTermsDialogProps = {
  openTermsDialog: boolean
  setOpenTermsDialog: (isOpen: boolean) => void
  creditFacility: NonNullable<GetCreditFacilityLayoutDetailsQuery["creditFacility"]>
}

export const CreditFacilityTermsDialog: React.FC<CreditFacilityTermsDialogProps> = ({
  openTermsDialog,
  setOpenTermsDialog,
  creditFacility,
}) => {
  const t = useTranslations("CreditFacilities.CreditFacilityDetails.TermsDialog")

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
      value: `${creditFacility.creditFacilityTerms.initialCvl}%`,
    },
    {
      label: t("details.marginCallCvl"),
      value: `${creditFacility.creditFacilityTerms.marginCallCvl}%`,
    },
    {
      label: t("details.liquidationCvl"),
      value: `${creditFacility.creditFacilityTerms.liquidationCvl}%`,
    },
    {
      label: t("details.dateCreated"),
      value: formatDate(creditFacility.createdAt),
    },
    {
      label: t("details.structuringFeeRate"),
      value: `${creditFacility.creditFacilityTerms.oneTimeFeeRate}%`,
    },
  ]

  return (
    <Dialog open={openTermsDialog} onOpenChange={setOpenTermsDialog}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
        </DialogHeader>
        <div className="py-2">
          <DetailsCard columns={2} variant="container" details={details} />
        </div>
      </DialogContent>
    </Dialog>
  )
}
