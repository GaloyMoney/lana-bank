"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { useLiquidationEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query LiquidationEventHistory($liquidationId: UUID!, $first: Int!, $after: String) {
    liquidation(id: $liquidationId) {
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

type LiquidationEventHistoryProps = {
  liquidationId: string
}

export const LiquidationEventHistory: React.FC<LiquidationEventHistoryProps> = ({
  liquidationId,
}) => {
  const { data, loading } = useLiquidationEventHistoryQuery({
    variables: { liquidationId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Liquidations.LiquidationDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.liquidation"
      events={data?.liquidation?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
