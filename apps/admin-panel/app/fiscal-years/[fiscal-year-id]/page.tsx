"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import FiscalYearDetailsCard from "./details"
import MonthClosuresList from "./month-closures-list"

import { useGetFiscalYearDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  fragment FiscalYearDetailsPageFragment on FiscalYear {
    id
    fiscalYearId
    chartId
    openedAsOf
    isOpen
    isLastMonthOfYearClosed
    reference
    nextMonthToClose
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
  const tCommon = useTranslations("Common")

  const { data, loading, error } = useGetFiscalYearDetailsQuery({
    variables: { fiscalYearId },
  })

  if (loading) {
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
