"use client"

import React from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DEFAULT_PAGESIZE, PaginatedData } from "@/components/paginated-table"
import { EntityEventHistory } from "@/components/entity-event-history"
import { useRoleEventHistoryQuery, EventTimelineEntry } from "@/lib/graphql/generated"

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
            userId
            payload
          }
        }
        pageInfo {
          endCursor
          startCursor
          hasNextPage
          hasPreviousPage
        }
      }
    }
  }
`

type RoleEventHistoryProps = {
  roleId: string
}

export const RoleEventHistory: React.FC<RoleEventHistoryProps> = ({ roleId }) => {
  const t = useTranslations("RolesAndPermissions.eventHistory")

  const { data, loading, fetchMore } = useRoleEventHistoryQuery({
    variables: { id: roleId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      title={t("title")}
      description={t("description")}
      emptyMessage={t("emptyMessage")}
      translationNamespace="EntityEvents.role"
      data={
        data?.role?.eventHistory as PaginatedData<EventTimelineEntry> | undefined
      }
      loading={loading}
      fetchMore={async (cursor) =>
        fetchMore({
          variables: { after: cursor },
        })
      }
    />
  )
}
