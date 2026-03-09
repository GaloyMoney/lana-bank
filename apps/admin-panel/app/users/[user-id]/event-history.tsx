"use client"

import React from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DEFAULT_PAGESIZE, PaginatedData } from "@/components/paginated-table"
import { EntityEventHistory } from "@/components/entity-event-history"
import { useUserEventHistoryQuery, EventTimelineEntry } from "@/lib/graphql/generated"

gql`
  query UserEventHistory($id: UUID!, $first: Int!, $after: String) {
    user(id: $id) {
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

type UserEventHistoryProps = {
  userId: string
}

export const UserEventHistory: React.FC<UserEventHistoryProps> = ({ userId }) => {
  const t = useTranslations("Users.eventHistory")

  const { data, loading, fetchMore } = useUserEventHistoryQuery({
    variables: { id: userId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      title={t("title")}
      description={t("description")}
      emptyMessage={t("emptyMessage")}
      translationNamespace="EntityEvents.user"
      data={
        data?.user?.eventHistory as PaginatedData<EventTimelineEntry> | undefined
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
