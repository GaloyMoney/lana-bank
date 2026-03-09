"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
import { usePolicyEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query PolicyEventHistory($id: UUID!, $first: Int!, $after: String) {
    policy(id: $id) {
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

type PolicyEventHistoryProps = {
  policyId: string
}

export const PolicyEventHistory: React.FC<PolicyEventHistoryProps> = ({
  policyId,
}) => {
  const { data, loading, fetchMore } = usePolicyEventHistoryQuery({
    variables: { id: policyId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="Policies.PolicyDetails.eventHistory"
      data={data?.policy?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
