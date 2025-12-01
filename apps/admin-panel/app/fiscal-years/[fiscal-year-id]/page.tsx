"use client"

import { gql } from "@apollo/client"
import { use, useEffect } from "react"
import { useTranslations } from "next-intl"

import FiscalYearDetailsCard from "./details"
import MonthClosuresList from "./month-closures-list"

import { useGetFiscalYearDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"
import { useBreadcrumb } from "@/app/breadcrumb-provider"

gql`
  fragment FiscalYearDetailsPageFragment on FiscalYear {
    id
    fiscalYearId
    chartId
    openedAsOf
    isOpen
    monthClosures {
      closedAsOf
      closedAt
    }
  }

  query GetFiscalYearDetails($fiscalYearId: UUID!) {
    fiscalYear(fiscalYearId: $fiscalYearId) {
      ...FiscalYearDetailsPageFragment
    }
  }
`

function FiscalYearPage({
  params,
}: {
  params: Promise<{
    "fiscal-year-id": string
  }>
}) {
  const { "fiscal-year-id": fiscalYearId } = use(params)
  const { setCustomLinks, resetToDefault } = useBreadcrumb()
  const navTranslations = useTranslations("Sidebar.navItems")
  const tCommon = useTranslations("Common")

  const { data, loading, error } = useGetFiscalYearDetailsQuery({
    variables: { fiscalYearId },
  })

  useEffect(() => {
    if (data?.fiscalYear) {
      setCustomLinks([
        { title: navTranslations("fiscalYears"), href: "/fiscal-years" },
        {
          title: fiscalYearId,
          isCurrentPage: true,
        },
      ])
    }
    return () => {
      resetToDefault()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [data?.fiscalYear])

  if (loading && !data) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.fiscalYear) return <div>{tCommon("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <FiscalYearDetailsCard fiscalYear={data.fiscalYear} />
      <MonthClosuresList monthClosures={data.fiscalYear.monthClosures} />
    </main>
  )
}

export default FiscalYearPage
