"use client"
import React, { useEffect, use } from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import TermsTemplateDetailsCard from "./details"

import { useBreadcrumb } from "@/app/breadcrumb-provider"
import { useTermsTemplateQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useCreateContext } from "@/app/create"

gql`
  query TermsTemplate($id: UUID!) {
    termsTemplate(termsTemplateId: $id) {
      ...TermsTemplateFields
    }
  }
`

function TermsTemplatePage({
  params,
}: {
  params: Promise<{
    "terms-template-id": string
  }>
}) {
  const { "terms-template-id": termsTemplateId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const { setTermsTemplate } = useCreateContext()
  const navTranslations = useTranslations("Sidebar.navItems")
  const { data, loading, error } = useTermsTemplateQuery({
    variables: { id: termsTemplateId },
  })

  useEffect(() => {
    if (data?.termsTemplate) {
      setCustomLinks([
        { title: navTranslations("termTemplates"), href: "/terms-templates" },
        { title: data.termsTemplate.name, isCurrentPage: true },
      ])
    }

    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.termsTemplate])

  useEffect(() => {
    data?.termsTemplate && setTermsTemplate(data?.termsTemplate)
    return () => setTermsTemplate(null)
  }, [data?.termsTemplate, setTermsTemplate])

  if (loading && !data)
    return <DetailsPageSkeleton tabs={0} detailItems={8} tabsCards={0} />
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.termsTemplate) return <div>Not found</div>

  return (
    <main className="max-w-7xl m-auto">
      <TermsTemplateDetailsCard termsTemplate={data.termsTemplate} />
    </main>
  )
}

export default TermsTemplatePage
