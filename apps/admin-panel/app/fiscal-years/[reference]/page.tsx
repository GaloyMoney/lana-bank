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

  query GetFiscalYearDetails($reference: String!) {
    fiscalYearByReference(reference: $reference) {
      ...FiscalYearDetailsPageFragment
    }
  }
`

function FiscalYearPage({
  params,
}: {
  params: Promise<{
    reference: string
  }>
}) {
  const { reference } = use(params)
  const tCommon = useTranslations("Common")

  const { data, loading, error } = useGetFiscalYearDetailsQuery({
    variables: { reference },
  })

  if (loading) {
    return <DetailsPageSkeleton tabs={0} tabsCards={0} />
  }
  if (error) return <div className="text-destructive">{error.message}</div>
  if (!data?.fiscalYearByReference) return <div>{tCommon("notFound")}</div>

  return (
    <main className="max-w-7xl m-auto space-y-2">
      <FiscalYearDetailsCard fiscalYear={data.fiscalYearByReference} />
      <MonthClosuresList monthClosures={data.fiscalYearByReference.monthClosures} />
    </main>
  )
}

export default FiscalYearPage
