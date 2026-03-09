"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { useWithdrawalEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query WithdrawalEventHistory($id: PublicId!, $first: Int!, $after: String) {
    withdrawalByPublicId(id: $id) {
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

type WithdrawalEventHistoryProps = {
  withdrawalPublicId: string
}

export const WithdrawalEventHistory: React.FC<WithdrawalEventHistoryProps> = ({
  withdrawalPublicId,
}) => {
  const { data, loading, fetchMore } = useWithdrawalEventHistoryQuery({
    variables: { id: withdrawalPublicId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Withdrawals.WithdrawDetails.eventHistory"
      data={data?.withdrawalByPublicId?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
