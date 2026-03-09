"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
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

type DisbursalEventHistoryProps = {
  publicId: string
}

export const DisbursalEventHistory: React.FC<DisbursalEventHistoryProps> = ({
  publicId,
}) => {
  const { data, loading } = useDisbursalEventHistoryQuery({
    variables: { publicId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Disbursals.DisbursalDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.disbursal"
      events={data?.disbursalByPublicId?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
