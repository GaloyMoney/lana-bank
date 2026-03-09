"use client"

import React from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { formatDate } from "@lana/web/utils"

import CardWrapper from "@/components/card-wrapper"
import DataTable, { Column } from "@/components/data-table"
import { EventPayload } from "@/components/event-payload"
import {
  useCustomerEventHistoryQuery,
  EventTimelineEntry,
} from "@/lib/graphql/generated"

gql`
  query CustomerEventHistory($id: PublicId!, $first: Int!, $after: String) {
    customerByPublicId(id: $id) {
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

type CustomerEventHistoryProps = {
  customerId: string
}

export const CustomerEventHistory: React.FC<CustomerEventHistoryProps> = ({
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.eventHistory")
  const te = useTranslations("EntityEvents.customer")

  const { data, loading } = useCustomerEventHistoryQuery({
    variables: { id: customerId, first: 100 },
  })

  const events = data?.customerByPublicId?.eventHistory.nodes ?? []

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
