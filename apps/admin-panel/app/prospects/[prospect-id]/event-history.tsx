"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
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

type ProspectEventHistoryProps = {
  prospectId: string
}

export const ProspectEventHistory: React.FC<ProspectEventHistoryProps> = ({
  prospectId,
}) => {
  const { data, loading } = useProspectEventHistoryQuery({
    variables: { id: prospectId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Prospects.ProspectDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.prospect"
      events={data?.prospectByPublicId?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
