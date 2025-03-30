"use client"

import { useTranslations } from "next-intl"
import React, { useState } from "react"
import { gql } from "@apollo/client"

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@lana/web/ui/card"

import DataTable, { Column } from "../../components/data-table"

import { TermsTemplate, useTermsTemplatesQuery } from "@/lib/graphql/generated"
import { formatPeriod } from "@/lib/utils"
import { UpdateTermsTemplateDialog } from "@/app/terms-templates/[terms-template-id]/update"

gql`
  fragment TermsTemplateFields on TermsTemplate {
    id
    name
    termsId
    createdAt
    subjectCanUpdateTermsTemplate
    values {
      annualRate
      accrualCycleInterval
      accrualInterval
      liquidationCvl
      marginCallCvl
      initialCvl
      oneTimeFeeRate
      duration {
        period
        units
      }
    }
  }

  query TermsTemplates {
    termsTemplates {
      ...TermsTemplateFields
    }
  }
`

const columns = (t: ReturnType<typeof useTranslations>): Column<TermsTemplate>[] => [
  {
    key: "name",
    header: t("table.headers.name"),
  },
  {
    key: "values",
    header: t("table.headers.duration"),
    render: (values) =>
      `${String(values.duration.units)} ${formatPeriod(values.duration.period)}`,
  },
  {
    key: "values",
    header: t("table.headers.annualRate"),
    render: (values) => `${values.annualRate}%`,
  },
  {
    key: "values",
    header: t("table.headers.initialCvl"),
    render: (values) => `${values.initialCvl}%`,
  },
  {
    key: "values",
    header: t("table.headers.marginCallCvl"),
    render: (values) => `${values.marginCallCvl}%`,
  },
  {
    key: "values",
    header: t("table.headers.liquidationCvl"),
    render: (values) => `${values.liquidationCvl}%`,
  },
]

function TermPage() {
  const t = useTranslations("TermsTemplates")

  const { data, loading, error } = useTermsTemplatesQuery()
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState<TermsTemplate | null>(null)

  if (error) {
    return (
      <Card>
        <CardContent>
          <p className="text-destructive mt-6">{t("errors.general")}</p>
        </CardContent>
      </Card>
    )
  }

  return (
    <main>
      {openUpdateTermsTemplateDialog && (
        <UpdateTermsTemplateDialog
          termsTemplate={openUpdateTermsTemplateDialog}
          openUpdateTermsTemplateDialog={Boolean(openUpdateTermsTemplateDialog)}
          setOpenUpdateTermsTemplateDialog={() => setOpenUpdateTermsTemplateDialog(null)}
        />
      )}
      <Card>
        <CardHeader>
          <CardTitle>{t("title")}</CardTitle>
          <CardDescription>{t("description")}</CardDescription>
        </CardHeader>
        <CardContent>
          <DataTable
            data={data?.termsTemplates || []}
            columns={columns(t)}
            loading={loading}
            navigateTo={(template) => `/terms-templates/${template.termsId}`}
          />
        </CardContent>
      </Card>
    </main>
  )
}

export default TermPage
