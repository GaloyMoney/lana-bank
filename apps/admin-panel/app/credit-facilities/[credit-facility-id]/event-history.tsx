"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useCreditFacilityEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query CreditFacilityEventHistory($publicId: PublicId!, $first: Int!, $after: String) {
    creditFacilityByPublicId(id: $publicId) {
      id
      eventHistory(first: $first, after: $after) {
        ...EventHistoryConnectionFields
      }
    }
  }
`

type CreditFacilityEventHistoryProps = {
  publicId: string
}

export const CreditFacilityEventHistory: React.FC<CreditFacilityEventHistoryProps> = ({
  publicId,
}) => {
  const { data, loading, fetchMore } = useCreditFacilityEventHistoryQuery({
    variables: { publicId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="CreditFacilities.CreditFacilityDetails.eventHistory"
      data={data?.creditFacilityByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
