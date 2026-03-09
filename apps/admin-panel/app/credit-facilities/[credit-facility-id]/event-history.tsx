"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { useCreditFacilityEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query CreditFacilityEventHistory($publicId: PublicId!, $first: Int!, $after: String) {
    creditFacilityByPublicId(id: $publicId) {
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

type CreditFacilityEventHistoryProps = {
  publicId: string
}

export const CreditFacilityEventHistory: React.FC<CreditFacilityEventHistoryProps> = ({
  publicId,
}) => {
  const { data, loading } = useCreditFacilityEventHistoryQuery({
    variables: { publicId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="CreditFacilities.CreditFacilityDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.creditFacility"
      events={data?.creditFacilityByPublicId?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
