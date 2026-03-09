"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useDisbursalEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query DisbursalEventHistory($publicId: PublicId!, $first: Int!, $after: String) {
    disbursalByPublicId(id: $publicId) {
      id
      eventHistory(first: $first, after: $after) {
        edges {
          cursor
          node {
            eventType
            recordedAt
            sequence
            auditEntryId
            subject {
              ... on User { userId, email }
              ... on System { actor }
            }
            payload
          }
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
        }
      }
    }
  }
`

type DisbursalEventHistoryProps = {
  publicId: string
}

export const DisbursalEventHistory: React.FC<DisbursalEventHistoryProps> = ({
  publicId,
}) => {
  const { data, loading, fetchMore } = useDisbursalEventHistoryQuery({
    variables: { publicId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Disbursals.DisbursalDetails.eventHistory"
      data={data?.disbursalByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
