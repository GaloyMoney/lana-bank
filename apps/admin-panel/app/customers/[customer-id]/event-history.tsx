"use client"

import React from "react"
import { gql } from "@apollo/client"
import { useTranslations } from "next-intl"

import { DEFAULT_PAGESIZE, PaginatedData } from "@/components/paginated-table"
import { EntityEventHistory } from "@/components/entity-event-history"
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

type CustomerEventHistoryProps = {
  customerId: string
}

export const CustomerEventHistory: React.FC<CustomerEventHistoryProps> = ({
  customerId,
}) => {
  const t = useTranslations("Customers.CustomerDetails.eventHistory")

  const { data, loading, fetchMore } = useCustomerEventHistoryQuery({
    variables: { id: customerId, first: DEFAULT_PAGESIZE },
  })

  return (
    <EntityEventHistory
      title={t("title")}
      description={t("description")}
      emptyMessage={t("emptyMessage")}
      translationNamespace="EntityEvents.customer"
      data={
        data?.customerByPublicId?.eventHistory as
          | PaginatedData<EventTimelineEntry>
          | undefined
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
