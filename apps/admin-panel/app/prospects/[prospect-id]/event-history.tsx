"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useProspectEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query ProspectEventHistory($id: PublicId!, $first: Int!, $after: String) {
    prospectByPublicId(id: $id) {
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

type ProspectEventHistoryProps = {
  prospectId: string
}

export const ProspectEventHistory: React.FC<ProspectEventHistoryProps> = ({
  prospectId,
}) => {
  const { data, loading, fetchMore } = useProspectEventHistoryQuery({
    variables: { id: prospectId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Prospects.ProspectDetails.eventHistory"
      data={data?.prospectByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
