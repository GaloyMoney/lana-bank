"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
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
        nodes {
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
        pageInfo {
          hasNextPage
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
  const { data, loading } = useCommitteeEventHistoryQuery({
    variables: { id: committeeId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Committees.CommitteeDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.committee"
      events={data?.committee?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
