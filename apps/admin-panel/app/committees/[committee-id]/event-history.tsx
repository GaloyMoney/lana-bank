"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useCommitteeEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query CommitteeEventHistory($id: UUID!, $first: Int!, $after: String) {
    committee(id: $id) {
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

type CommitteeEventHistoryProps = {
  committeeId: string
}

export const CommitteeEventHistory: React.FC<CommitteeEventHistoryProps> = ({
  committeeId,
}) => {
  const { data, loading, fetchMore } = useCommitteeEventHistoryQuery({
    variables: { id: committeeId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Committees.CommitteeDetails.eventHistory"
      data={data?.committee?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
