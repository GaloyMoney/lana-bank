"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { usePendingCreditFacilityEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query PendingCreditFacilityEventHistory($id: UUID!, $first: Int!, $after: String) {
    pendingCreditFacility(id: $id) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type PendingCreditFacilityEventHistoryProps = {
  pendingId: string
}

export const PendingCreditFacilityEventHistory: React.FC<
  PendingCreditFacilityEventHistoryProps
> = ({ pendingId }) => {
  const { data, loading, fetchMore } = usePendingCreditFacilityEventHistoryQuery({
    variables: { id: pendingId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="PendingCreditFacilities.PendingDetails.eventHistory"
      data={data?.pendingCreditFacility?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
