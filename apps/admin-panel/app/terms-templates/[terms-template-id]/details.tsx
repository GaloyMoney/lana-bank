"use client"
import { useTranslations } from "next-intl"
import React, { useState } from "react"

import { Button } from "@lana/web/ui/button"

import { formatDate } from "@lana/web/utils"

import { TermsTemplateQuery } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { UpdateTermsTemplateDialog } from "@/app/terms-templates/[terms-template-id]/update"

type TermsTemplateDetailsProps = {
  termsTemplate: NonNullable<TermsTemplateQuery["termsTemplate"]>
}

const TermsTemplateDetailsCard: React.FC<TermsTemplateDetailsProps> = ({
  termsTemplate,
}) => {
  const t = useTranslations("TermsTemplates.TermsTemplateDetails.DetailsCard")

  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState(false)

  const details: DetailItemProps[] = [
    { label: t("fields.name"), value: termsTemplate.name },
    { label: t("fields.createdAt"), value: formatDate(termsTemplate.createdAt) },
    {
      label: t("fields.duration"),
      value: (
        <>
          {termsTemplate.values.duration.units}{" "}
          <PeriodLabel period={termsTemplate.values.duration.period} />
        </>
      ),
    },
    {
      label: t("fields.annualRate"),
      value: `${termsTemplate.values.annualRate}%`,
    },
    {
      label: t("fields.initialCvl"),
      value: `${termsTemplate.values.initialCvl}%`,
    },
    {
      label: t("fields.marginCallCvl"),
      value: `${termsTemplate.values.marginCallCvl}%`,
    },
    {
      label: t("fields.liquidationCvl"),
      value: `${termsTemplate.values.liquidationCvl}%`,
    },
    {
      label: t("fields.oneTimeFeeRate"),
      value: `${termsTemplate.values.oneTimeFeeRate}%`,
    },
  ]

  const footerContent = (
    <Button
      variant="outline"
      onClick={() => setOpenUpdateTermsTemplateDialog(true)}
      data-testid="terms-template-update-button"
    >
      {t("buttons.update")}
    </Button>
  )

  return (
    <>
      <UpdateTermsTemplateDialog
        termsTemplate={termsTemplate}
        openUpdateTermsTemplateDialog={openUpdateTermsTemplateDialog}
        setOpenUpdateTermsTemplateDialog={setOpenUpdateTermsTemplateDialog}
      />

      <DetailsCard
        title={t("title")}
        details={details}
        footerContent={footerContent}
        className="w-full"
      />
    </>
  )
}

export default TermsTemplateDetailsCard
