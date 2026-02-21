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

import {
  TermsTemplateFieldsFragment,
  TermsTemplatesQuery,
  useTermsTemplatesQuery,
} from "@/lib/graphql/generated"
import { PeriodLabel } from "@/app/credit-facilities/label"
import { UpdateTermsTemplateDialog } from "@/app/terms-templates/[terms-template-id]/update"
import { formatCvl } from "@/lib/utils"

gql`
  fragment TermsTemplateFields on TermsTemplate {
    id
    name
    termsId
    createdAt
    userCanUpdateTermsTemplate
    values {
      annualRate
      liquidationCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
          isInfinite
        }
      }
      marginCallCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
          isInfinite
        }
      }
      initialCvl {
        __typename
        ... on FiniteCvlPct {
          value
        }
        ... on InfiniteCvlPct {
          isInfinite
        }
      }
      oneTimeFeeRate
      disbursalPolicy
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

const columns = (
  t: ReturnType<typeof useTranslations>,
): Column<NonNullable<TermsTemplatesQuery["termsTemplates"]>[number]>[] => [
  {
    key: "name",
    header: t("table.headers.name"),
    width: "25%",
  },
  {
    key: "values",
    header: t("table.headers.duration"),
    width: "15%",
    render: (values) => (
      <>
        {String(values.duration.units)} <PeriodLabel period={values.duration.period} />
      </>
    ),
  },
  {
    key: "values",
    header: t("table.headers.annualRate"),
    width: "12%",
    render: (values) => `${values.annualRate}%`,
  },
  {
    key: "values",
    header: t("table.headers.initialCvl"),
    width: "12%",
    render: (values) => formatCvl(values.initialCvl),
  },
  {
    key: "values",
    header: t("table.headers.marginCallCvl"),
    width: "15%",
    render: (values) => formatCvl(values.marginCallCvl),
  },
  {
    key: "values",
    header: t("table.headers.liquidationCvl"),
    width: "15%",
    render: (values) => formatCvl(values.liquidationCvl),
  },
]

function TermPage() {
  const t = useTranslations("TermsTemplates")

  const { data, loading, error } = useTermsTemplatesQuery()
  const [openUpdateTermsTemplateDialog, setOpenUpdateTermsTemplateDialog] =
    useState<TermsTemplateFieldsFragment | null>(null)

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
