"use client"

import { gql } from "@apollo/client"
import { use } from "react"
import { useTranslations } from "next-intl"

import { Tabs, TabsList, TabsTrigger, TabsContent } from "@lana/web/ui/tabs"

import FiscalYearDetailsCard from "./details"
import MonthClosuresList from "./month-closures-list"
import { FiscalYearEventHistory } from "./event-history"

import { useGetFiscalYearDetailsQuery } from "@/lib/graphql/generated"
import { DetailsPageSkeleton } from "@/components/details-page-skeleton"

gql`
  fragment FiscalYearDetailsPageFragment on FiscalYear {
    id
    fiscalYearId
    chartId
    openedAsOf
    createdAt
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
  const tTabs = useTranslations("FiscalYears.tabs")

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
      <Tabs defaultValue="monthClosures">
        <TabsList>
          <TabsTrigger value="monthClosures">{tTabs("monthClosures")}</TabsTrigger>
          <TabsTrigger value="events">{tTabs("events")}</TabsTrigger>
        </TabsList>
        <TabsContent value="monthClosures">
          <MonthClosuresList monthClosures={data.fiscalYearByYear.monthClosures} />
        </TabsContent>
        <TabsContent value="events">
          <FiscalYearEventHistory year={year} />
        </TabsContent>
      </Tabs>
    </main>
  )
}

export default FiscalYearPage
