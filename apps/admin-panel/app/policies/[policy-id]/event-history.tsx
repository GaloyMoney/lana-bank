"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
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

type PolicyEventHistoryProps = {
  policyId: string
}

export const PolicyEventHistory: React.FC<PolicyEventHistoryProps> = ({
  policyId,
}) => {
  const { data, loading } = usePolicyEventHistoryQuery({
    variables: { id: policyId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Policies.PolicyDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.policy"
      events={data?.policy?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
