"use client"
import React, { useState } from "react"

import { TermsTemplateQuery } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { formatDate, formatInterval, formatPeriod } from "@/lib/utils"
import { Button } from "@/ui/button"
import { UpdateTermsTemplateDialog } from "@/components/terms-template/update-dialog"

type TermsTemplateDetailsProps = {
  termsTemplate: NonNullable<TermsTemplateQuery["termsTemplate"]>
  refetch: () => void
}

const TermsTemplateDetailsCard: React.FC<TermsTemplateDetailsProps> = ({
  termsTemplate,
  refetch,
}) => {
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState(false)

  const details: DetailItemProps[] = [
    { label: "Name", value: termsTemplate.name },
    { label: "Created At", value: formatDate(termsTemplate.createdAt) },
    {
      label: "Duration",
      value: `${termsTemplate.values.duration.units} ${formatPeriod(
        termsTemplate.values.duration.period,
      )}`,
    },
    {
      label: "Accrual Interval",
      value: formatInterval(termsTemplate.values.accrualInterval),
    },
    {
      label: "Incurrence Interval",
      value: formatInterval(termsTemplate.values.incurrenceInterval),
    },
    {
      label: "Annual Rate",
      value: `${termsTemplate.values.annualRate}%`,
    },
    {
      label: "Initial CVL",
      value: `${termsTemplate.values.initialCvl}%`,
    },
    {
      label: "Margin Call CVL",
      value: `${termsTemplate.values.marginCallCvl}%`,
    },
    {
      label: "Liquidation CVL",
      value: `${termsTemplate.values.liquidationCvl}%`,
    },
    {
      label: "One Time Fee Rate",
      value: `${termsTemplate.values.oneTimeFeeRate}%`,
    },
  ]

  const footerContent = (
    <Button variant="outline" onClick={() => setOpenUpdateTermsTemplateDialog(true)}>
      Update
    </Button>
  )

  return (
    <>
      <UpdateTermsTemplateDialog
        termsTemplate={termsTemplate}
        openUpdateTermsTemplateDialog={openUpdateTermsTemplateDialog}
        setOpenUpdateTermsTemplateDialog={setOpenUpdateTermsTemplateDialog}
        refetch={refetch}
      />

      <DetailsCard
        title="Terms Template"
        details={details}
        footerContent={footerContent}
        className="w-full"
      />
    </>
  )
}

export default TermsTemplateDetailsCard
