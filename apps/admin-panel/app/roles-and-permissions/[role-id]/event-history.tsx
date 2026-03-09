"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { useRoleEventHistoryQuery } from "@/lib/graphql/generated"

gql`
  query RoleEventHistory($id: UUID!, $first: Int!, $after: String) {
    role(id: $id) {
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

type RoleEventHistoryProps = {
  roleId: string
}

export const RoleEventHistory: React.FC<RoleEventHistoryProps> = ({ roleId }) => {
  const { data, loading } = useRoleEventHistoryQuery({
    variables: { id: roleId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="RolesAndPermissions.eventHistory"
      eventTranslationNamespace="EntityEvents.role"
      events={data?.role?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
