"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { DEFAULT_PAGESIZE } from "@/components/paginated-table"
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

type RoleEventHistoryProps = {
  roleId: string
}

export const RoleEventHistory: React.FC<RoleEventHistoryProps> = ({ roleId }) => {
  const { data, loading, fetchMore } = useRoleEventHistoryQuery({
    variables: { id: roleId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      translationNamespace="RolesAndPermissions.eventHistory"
      data={data?.role?.eventHistory}
      loading={loading}
      fetchMore={async (cursor) => fetchMore({ variables: { after: cursor } })}
    />
  )
}
