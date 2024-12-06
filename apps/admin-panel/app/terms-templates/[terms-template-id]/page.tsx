"use client"
import React, { useEffect } from "react"
import { gql } from "@apollo/client"

import TermsTemplateDetailsCard from "./details"

import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { useTermsTemplateQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  query TermsTemplate($id: UUID!) {
    termsTemplate(id: $id) {
      id
      name
      termsId
      createdAt
      subjectCanUpdateTermsTemplate
      values {
        duration {
          units
          period
        }
        accrualInterval
        incurrenceInterval
        annualRate
        initialCvl
        marginCallCvl
        liquidationCvl
        oneTimeFeeRate
      }
    }
  }
`

function TermsTemplatePage({
  params,
}: {
  params: {
    "terms-template-id": string
  }
}) {
  const { "terms-template-id": termsTemplateId } = params
  const { setCustomLinks, resetToDefault } = useBreadcrumb()

  const { data, loading, error, refetch } = useTermsTemplateQuery({
    variables: { id: termsTemplateId },
  })

  useEffect(() => {
    if (data?.termsTemplate) {
      setCustomLinks([
        { title: "Dashboard", href: "/dashboard" },
        { title: "Terms Templates", href: "/terms-templates" },
        { title: data.termsTemplate.name, isCurrentPage: true },
      ])
    }

    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.termsTemplate])

  if (loading) return <DetailsPageSkeleton tabs={0} detailItems={8} tabsCards={0} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.termsTemplate) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <TermsTemplateDetailsCard termsTemplate={data.termsTemplate} refetch={refetch} />
    </main>
  )
}

export default TermsTemplatePage
