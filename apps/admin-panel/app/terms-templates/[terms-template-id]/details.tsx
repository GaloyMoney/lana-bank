"use client"
import { useTranslations } from "next-intl"
import React, { useState } from "react"

import { Button } from "@lana/web/ui/button"
import { Copy } from "lucide-react"

import { formatDate } from "@lana/web/utils"

import { TermsTemplateQuery } from "@/lib/graphql/generated"
import { DetailsCard, DetailItemProps } from "@/components/details"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { UpdateTermsTemplateDialog } from "@/app/terms-templates/[terms-template-id]/update"
import { CreateTermsTemplateDialog } from "@/app/terms-templates/create"
import { formatCvl } from "@/lib/utils"

type TermsTemplateDetailsProps = {
  termsTemplate: NonNullable<TermsTemplateQuery["termsTemplate"]>
}

const TermsTemplateDetailsCard: React.FC<TermsTemplateDetailsProps> = ({
  termsTemplate,
}) => {
  const t = useTranslations("TermsTemplates.TermsTemplateDetails.DetailsCard")

  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState(false)
  const [openCreateTermsTemplateDialog, setOpenCreateTermsTemplateDialog] =
    useState(false)

  const disbursalPolicyLabel =
    termsTemplate.values.disbursalPolicy === "SINGLE_DISBURSAL"
      ? t("fields.singleDisbursal")
      : t("fields.multipleDisbursal")

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
      value: formatCvl(termsTemplate.values.initialCvl),
    },
    {
      label: t("fields.marginCallCvl"),
      value: formatCvl(termsTemplate.values.marginCallCvl),
    },
    {
      label: t("fields.liquidationCvl"),
      value: formatCvl(termsTemplate.values.liquidationCvl),
    },
    {
      label: t("fields.oneTimeFeeRate"),
      value: `${termsTemplate.values.oneTimeFeeRate}%`,
    },
    {
      label: t("fields.disbursalPolicy"),
      value: disbursalPolicyLabel,
    },
  ]

  const footerContent = (
    <div className="flex gap-2">
      <Button
        variant="outline"
        onClick={() => setOpenCreateTermsTemplateDialog(true)}
        data-testid="terms-template-duplicate-button"
      >
        <Copy className="h-4 w-4 mr-2" />
        {t("buttons.duplicate")}
      </Button>
      <Button
        variant="outline"
        onClick={() => setOpenUpdateTermsTemplateDialog(true)}
        data-testid="terms-template-update-button"
      >
        {t("buttons.update")}
      </Button>
    </div>
  )

  return (
    <>
      <UpdateTermsTemplateDialog
        termsTemplate={termsTemplate}
        openUpdateTermsTemplateDialog={openUpdateTermsTemplateDialog}
        setOpenUpdateTermsTemplateDialog={setOpenUpdateTermsTemplateDialog}
      />

      <CreateTermsTemplateDialog
        openCreateTermsTemplateDialog={openCreateTermsTemplateDialog}
        setOpenCreateTermsTemplateDialog={setOpenCreateTermsTemplateDialog}
        templateToDuplicate={termsTemplate}
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
