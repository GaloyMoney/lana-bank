"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useFiscalYearEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query FiscalYearEventHistory($year: String!, $first: Int!, $after: String) {
    fiscalYearByYear(year: $year) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type FiscalYearEventHistoryProps = {
  year: string
}

export const FiscalYearEventHistory: React.FC<FiscalYearEventHistoryProps> = ({
  year,
}) => {
  const { data, loading, fetchMore } = useFiscalYearEventHistoryQuery({
    variables: { year, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="FiscalYears.eventHistory"
      data={data?.fiscalYearByYear?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
