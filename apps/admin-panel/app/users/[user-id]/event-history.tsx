"use client"

import React from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"
import { EventPayload } from "@/components/event-payload"
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
        nodes {
          eventType
          recordedAt
          sequence
          auditEntryId
          userId
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

type UserEventHistoryProps = {
  userId: string
}

export const UserEventHistory: React.FC<UserEventHistoryProps> = ({ userId }) => {
  const t = useTranslations("Users.eventHistory")
  const te = useTranslations("EntityEvents.user")

  const { data, loading } = useUserEventHistoryQuery({
    variables: { id: userId, first: 100 },
  })

  const events = data?.user?.eventHistory.nodes ?? []

  const translateEventType = (eventType: string): string => {
    const key = eventType.toLowerCase()
    if (te.has(key)) {
      return te(key)
    }
    return eventType
  }

  const columns: Column<EventTimelineEntry>[] = [
    {
      key: "eventType",
      header: t("columns.event"),
      render: (eventType: string) => translateEventType(eventType),
    },
    {
      key: "payload",
      header: t("columns.details"),
      render: (payload: Record<string, unknown>) => <EventPayload payload={payload} />,
    },
    {
      key: "recordedAt",
      header: t("columns.recordedAt"),
      render: (recordedAt: string) => formatDate(recordedAt),
    },
  ]

  return (
    <CardWrapper title={t("title")} description={t("description")}>
      <DataTable
        data={events}
        columns={columns}
        loading={loading}
        emptyMessage={t("emptyMessage")}
      />
    </CardWrapper>
  )
}
