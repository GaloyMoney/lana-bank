"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
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

type LiquidationEventHistoryProps = {
  liquidationId: string
}

export const LiquidationEventHistory: React.FC<LiquidationEventHistoryProps> = ({
  liquidationId,
}) => {
  const { data, loading, fetchMore } = useLiquidationEventHistoryQuery({
    variables: { liquidationId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Liquidations.LiquidationDetails.eventHistory"
      data={data?.liquidation?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
