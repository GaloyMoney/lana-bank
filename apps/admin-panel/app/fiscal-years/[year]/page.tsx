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
    year
    nextMonthToClose
    monthClosures {
      closedAsOf
      closedAt
    }
  }

  query GetFiscalYearDetails($year: String!) {
    fiscalYearByYear(year: $year) {
      ...FiscalYearDetailsPageFragment
    }
  }
`

function FiscalYearPage({
  params,
}: {
  params: Promise<{
    year: string
  }>
}) {
  const { year } = use(params)
  const tCommon = useTranslations("Common")

  const { data, loading, error } = useGetFiscalYearDetailsQuery({
    variables: { year },
  })

  if (loading) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.fiscalYearByYear) return <div>{tCommon("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <FiscalYearDetailsCard fiscalYear={data.fiscalYearByYear} />
      <MonthClosuresList monthClosures={data.fiscalYearByYear.monthClosures} />
    </main>
  )
}

export default FiscalYearPage
