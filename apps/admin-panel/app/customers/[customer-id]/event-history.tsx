"use client"

import React from "react"
import { gql } from "@apollo/client"

import { EntityEventHistory } from "@/components/entity-event-history"
import { useCustomerEventHistoryQuery } from "@/lib/graphql/generated"

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

type CustomerEventHistoryProps = {
  customerId: string
}

export const CustomerEventHistory: React.FC<CustomerEventHistoryProps> = ({
  customerId,
}) => {
  const { data, loading } = useCustomerEventHistoryQuery({
    variables: { id: customerId, first: 100 },
  })

  return (
    <EntityEventHistory
      translationNamespace="Customers.CustomerDetails.eventHistory"
      eventTranslationNamespace="EntityEvents.customer"
      events={data?.customerByPublicId?.eventHistory.nodes ?? []}
      loading={loading}
    />
  )
}
